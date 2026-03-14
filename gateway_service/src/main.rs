use async_trait::async_trait;
use bytes::Bytes;
use once_cell::sync::Lazy;
use pingora::prelude::*;
use pingora::prelude::RequestHeader;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_proxy::{ProxyHttp, Session};
use serde::Deserialize;
use std::{
    borrow::Cow,
    fs,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

// ─── Config Structs ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GatewayConfig {
    gateway: GatewaySettings,
    routes: Vec<RouteConfig>,
}

#[derive(Debug, Deserialize)]
struct GatewaySettings {
    listen: String,
}

#[derive(Debug, Deserialize)]
struct RouteConfig {
    name: String,
    prefix: String,
    #[serde(default = "default_true")]
    strip_prefix: bool,
    #[serde(default)]
    load_balance: LoadBalance,
    health_check: Option<HealthCheckConfig>,
    servers: Vec<ServerConfig>,
}

#[derive(Debug, Deserialize, Clone)]
struct HealthCheckConfig {
    path: String,
    #[serde(default = "default_hc_interval")]
    interval_ms: u64,
    #[serde(default = "default_hc_timeout")]
    timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    url: String,
    #[serde(default = "default_weight")]
    weight: u32,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum LoadBalance {
    #[default]
    RoundRobin,
}

fn default_true() -> bool { true }
fn default_weight() -> u32 { 1 }
fn default_hc_interval() -> u64 { 10_000 }
fn default_hc_timeout() -> u64 { 2_000 }

// ─── Runtime Structs ──────────────────────────────────────────────────────────

struct ServerEntry {
    url:     Arc<str>,
    host:    Arc<str>,
    port:    u16,
    healthy: Arc<AtomicBool>,
}

pub struct Route {
    name:         Arc<str>,
    prefix:       Arc<str>,
    strip_prefix: bool,
    servers:      Vec<ServerEntry>,
    health_check: Option<HealthCheckConfig>,
    rr_counter:   AtomicUsize,
}

impl Route {
    fn from_config(cfg: &RouteConfig) -> Self {
        let servers = cfg.servers.iter().map(|s| {
            let (host, port) = parse_backend_url(&s.url)
                .unwrap_or_else(|_| panic!("Invalid server URL: {}", s.url));
            ServerEntry {
                url:     Arc::from(s.url.as_str()),
                host:    Arc::from(host.as_str()),
                port,
                // No health_check = always healthy. With health_check = wait for first probe.
                healthy: Arc::new(AtomicBool::new(cfg.health_check.is_none())),
            }
        }).collect();

        Self {
            name:         Arc::from(cfg.name.as_str()),
            prefix:       Arc::from(cfg.prefix.as_str()),
            strip_prefix: cfg.strip_prefix,
            servers,
            health_check: cfg.health_check.clone(),
            rr_counter:   AtomicUsize::new(0),
        }
    }

    fn next_server(&self) -> Option<(&str, u16)> {
        // FIX #4: avoid Vec allocation — count healthy, then pick by index in one pass
        let healthy_count = self.servers
            .iter()
            .filter(|s| s.healthy.load(Ordering::Relaxed))
            .count();

        if healthy_count == 0 {
            return None;
        }

        // FIX #3: fetch_add already wraps atomically — no need for CAS fetch_update
        let idx = self.rr_counter.fetch_add(1, Ordering::Relaxed) % healthy_count;

        // Walk healthy servers to find the idx-th one
        self.servers
            .iter()
            .filter(|s| s.healthy.load(Ordering::Relaxed))
            .nth(idx)
            .map(|s| (s.host.as_ref(), s.port))
    }

    #[inline]
    fn matches(&self, path: &str) -> bool {
        let p = self.prefix.as_ref();
        path.starts_with(p)
            && (path.len() == p.len()
                || path.as_bytes().get(p.len()) == Some(&b'/'))
    }
}

// ─── CTX ──────────────────────────────────────────────────────────────────────

pub struct Ctx {
    route: Option<&'static Route>,
}

// ─── Statics ──────────────────────────────────────────────────────────────────

static CONFIG: Lazy<GatewayConfig> = Lazy::new(|| {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "gateway.yaml".to_string());

    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Could not read config: {}", path));

    serde_yaml::from_str(&contents)
        .unwrap_or_else(|e| panic!("Failed to parse config: {}", e))
});

static ROUTES: Lazy<Vec<Route>> = Lazy::new(|| {
    CONFIG.routes.iter().map(Route::from_config).collect()
});

// FIX #1: was `include_bytes!("swagger.html")` — file might not exist at compile time.
// Use a &'static str constant instead, then convert to Bytes once.
static SWAGGER_HTML_STR: &str = r##"<!DOCTYPE html>
<html>
<head>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui.css" />
    <title>Gateway - Swagger UI</title>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-bundle.js" crossorigin></script>
    <script>
        SwaggerUIBundle({
            urls: [
                { url: '/booking/openapi.json', name: 'Booking API' },
                { url: '/users/openapi.json',   name: 'Users API'   },
                { url: '/mcp_service/openapi.json', name: 'MCP API' }
            ],
            "urls.primaryName": "Booking API",
            dom_id: '#swagger-ui',
            layout: 'BaseLayout',
            deepLinking: true,
            presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset]
        });
    </script>
</body>
</html>"##;

// Bytes::from_static needs &'static [u8] — SWAGGER_HTML_STR.as_bytes() is &'static [u8] ✓
static SWAGGER_HTML: Lazy<Bytes> = Lazy::new(|| {
    Bytes::from_static(SWAGGER_HTML_STR.as_bytes())
});

// ─── Health Checks ────────────────────────────────────────────────────────────

fn make_http_client(timeout_ms: u64) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .tcp_keepalive(Duration::from_secs(30))
        .pool_max_idle_per_host(1)
        .build()
        .expect("Failed to build HTTP client")
}

// FIX #2 + #3: both args are &str — callers must pass .as_ref() for Arc<str>
fn probe(client: &reqwest::blocking::Client, url: &str, hc_path: &str) -> bool {
    let full = [url.trim_end_matches('/'), hc_path].concat();
    client
        .get(&full)
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn run_startup_health_checks() {
    println!("── Startup Health Checks ──────────────────────────");

    for route in ROUTES.iter() {
        match &route.health_check {
            None => println!(
                "  [{}] no health_check — {} server(s) trusted by default",
                route.name,
                route.servers.len()
            ),
            Some(hc) => {
                let client = make_http_client(hc.timeout_ms);
                let mut healthy_count = 0usize;

                for server in &route.servers {
                    // FIX #2: Arc<str> → &str via .as_ref()
                    let ok = probe(&client, server.url.as_ref(), &hc.path);
                    server.healthy.store(ok, Ordering::Relaxed);

                    if ok {
                        healthy_count += 1;
                        println!("  [{}] ✓  {} → healthy", route.name, server.url);
                    } else {
                        println!(
                            "  [{}] ✗  {} → failed ({}{}), skipping",
                            route.name, server.url, server.url, hc.path
                        );
                    }
                }

                if healthy_count == 0 {
                    println!(
                        "  [{}] ⚠  ALL servers unhealthy — will 503",
                        route.name
                    );
                } else {
                    println!(
                        "  [{}] {}/{} healthy",
                        route.name, healthy_count, route.servers.len()
                    );
                }
            }
        }
    }

    println!("────────────────────────────────────────────────────\n");
}

fn spawn_background_health_checks() {
    for route in ROUTES.iter() {
        if let Some(hc) = &route.health_check {
            for server in &route.servers {
                let url        = Arc::clone(&server.url);
                let hc_path    = hc.path.clone();         // String — cheap, only at startup
                let timeout    = hc.timeout_ms;
                let interval   = hc.interval_ms;
                let healthy    = Arc::clone(&server.healthy);
                let route_name = Arc::clone(&route.name);

                std::thread::spawn(move || {
                    let client = make_http_client(timeout); // built once, reused forever

                    loop {
                        std::thread::sleep(Duration::from_millis(interval));

                        // FIX #2: Arc<str> → &str via .as_ref()
                        let ok  = probe(&client, url.as_ref(), &hc_path);
                        let was = healthy.swap(ok, Ordering::Relaxed);

                        match (was, ok) {
                            (true, false) => println!(
                                "  [{}] ✗  {} → unhealthy, removed from load balancer",
                                route_name, url
                            ),
                            (false, true) => println!(
                                "  [{}] ✓  {} → recovered, added back to load balancer",
                                route_name, url
                            ),
                            _ => {} // no state change — silent
                        }
                    }
                });
            }
        }
    }
}

// ─── Gateway ──────────────────────────────────────────────────────────────────

pub struct Gateway;

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = Ctx;

    fn new_ctx(&self) -> Ctx {
        Ctx { route: None }
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Ctx,
    ) -> Result<bool> {
        let path = session.req_header().uri.path();

        if path == "/docs" || path == "/docs/" {
            let mut header = ResponseHeader::build(200, None)?;
            // FIX #5: insert_header can fail — use ? (pingora header result is compatible)
            header.insert_header("Content-Type", "text/html; charset=utf-8")?;
            header.insert_header("Content-Length", SWAGGER_HTML.len().to_string())?;
            session.write_response_header(Box::new(header), false).await?;
            session.write_response_body(Some(SWAGGER_HTML.clone()), true).await?;
            return Ok(true);
        }

        // Route matched ONCE here — stored in ctx, reused by both downstream methods
        ctx.route = find_route(path);
        Ok(false)
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Ctx,
    ) -> Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();

        let route = ctx.route.ok_or_else(|| {
            Error::explain(
                ErrorType::HTTPStatus(404),
                format!("No route for path: {}", path),
            )
        })?;

        let (host, port) = route.next_server().ok_or_else(|| {
            Error::explain(
                ErrorType::HTTPStatus(503),
                format!("All servers unhealthy for route: {}", route.name),
            )
        })?;

        Ok(Box::new(HttpPeer::new((host, port), false, "".to_string())))
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Ctx,
    ) -> Result<()> {
        let path  = session.req_header().uri.path();
        let query = session.req_header().uri.query().unwrap_or("");

        // Cow: zero allocation when no query string (the common case)
        let original_uri: Cow<str> = if query.is_empty() {
            Cow::Borrowed(path)
        } else {
            Cow::Owned(format!("{}?{}", path, query))
        };

        // FIX #7: if ctx.route is None here, something is wrong — return error
        let route = ctx.route.ok_or_else(|| {
            Error::explain(
                ErrorType::HTTPStatus(500),
                "upstream_request_filter called with no matched route".to_string(),
            )
        })?;

        let (prefix, new_path): (&str, Cow<str>) = if route.strip_prefix {
            let stripped = path
                .strip_prefix(route.prefix.as_ref())
                .unwrap_or("/");
            let stripped = if stripped.is_empty() { "/" } else { stripped };
            (route.prefix.as_ref(), Cow::Borrowed(stripped))
        } else {
            ("", Cow::Borrowed(path))
        };

        // Cow: only allocates when query string is present
        let new_uri: Cow<str> = if query.is_empty() {
            new_path
        } else {
            Cow::Owned(format!("{}?{}", new_path, query))
        };

        // FIX #5: parse().map_err(...) instead of .unwrap() — never panics in prod
        upstream_request.set_uri(
            new_uri.parse().map_err(|e| {
                Error::explain(
                    ErrorType::InvalidHTTPHeader,
                    format!("Failed to parse stripped URI '{}': {}", new_uri, e),
                )
            })?
        );

        // These header values are derived from the request path — already HTTP-validated.
        // Using map_err to stay in Result<()> rather than unwrap.
        upstream_request
            .insert_header("X-Original-URI", original_uri.as_ref())
            .map_err(|e| Error::explain(ErrorType::InvalidHTTPHeader, e.to_string()))?;

        upstream_request
            .insert_header("X-Original-Path", path)
            .map_err(|e| Error::explain(ErrorType::InvalidHTTPHeader, e.to_string()))?;

        upstream_request
            .insert_header("X-Forwarded-Prefix", prefix)
            .map_err(|e| Error::explain(ErrorType::InvalidHTTPHeader, e.to_string()))?;

        Ok(())
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

#[inline]
fn find_route(path: &str) -> Option<&'static Route> {
    ROUTES.iter().find(|r| r.matches(path))
}

fn parse_backend_url(url: &str) -> pingora::Result<(String, u16)> {
    let stripped = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);

    // rsplit_once correctly handles IPv6 like [::1]:8080
    let (host, port_str) = stripped.rsplit_once(':').ok_or_else(|| {
        Error::explain(
            ErrorType::InvalidHTTPHeader,
            format!("Missing port in URL: {}", url),
        )
    })?;

    let port = port_str.parse::<u16>().map_err(|e| {
        Error::explain(
            ErrorType::InvalidHTTPHeader,
            format!("Invalid port in '{}': {}", url, e),
        )
    })?;

    // Strip IPv6 brackets: [::1] → ::1
    let host = host.trim_matches(|c| c == '[' || c == ']').to_string();
    Ok((host, port))
}

fn assert_port_available(addr: &str) {
    std::net::TcpListener::bind(addr)
        .unwrap_or_else(|e| panic!("Port unavailable {}: {}", addr, e));
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    println!("Loaded {} route(s)\n", ROUTES.len());

    run_startup_health_checks();
    spawn_background_health_checks();

    assert_port_available(&CONFIG.gateway.listen);

    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let mut svc = pingora_proxy::http_proxy_service(&server.configuration, Gateway);
    svc.add_tcp(&CONFIG.gateway.listen);
    server.add_service(svc);

    println!("Listening on {}\n", CONFIG.gateway.listen);
    server.run_forever();
}