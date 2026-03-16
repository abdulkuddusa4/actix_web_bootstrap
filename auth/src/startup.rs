use std::net::TcpListener;
use actix_web::{App, Error, HttpServer, middleware::Logger, web};
use env_logger::Env;
use lettre::transport::smtp::authentication::Credentials;
use sea_orm::DatabaseConnection;
use utoipa::{Modify, OpenApi, openapi::{self, security::{HttpAuthScheme, HttpBuilder, SecurityScheme}}};
use utoipa_swagger_ui::SwaggerUi;

use crate::accounts::get_router as get_accounts_router;




struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}


#[derive(OpenApi)]
#[openapi(
    // 👇 nest mirrors .service(web::scope("/accounts"))
    nest(
        (path = "/accounts", api = crate::accounts::AccountsApiDoc),
        // add more modules here as your app grows:
        // (path = "/orders", api = orders::OrdersApiDoc),
        // (path = "/auth",   api = auth::AuthApiDoc),
    ),
    modifiers(&SecurityAddon), 
    servers(
        // (url = "/accounts", description = "Production (behind proxy)"),
        // (url = "/",    description = "Local dev"),
    ),

    info(title = "My API", version = "1.0.0")
)]
struct ApiDoc;


struct Config{
    redis_client: deadpool_redis::Pool,
    db: DatabaseConnection,
    mail_cred: Credentials
}


pub fn init_actix_web_server(
    listener: TcpListener,
    db: sea_orm::DatabaseConnection
)
-> Result<actix_web::dev::Server, String>
{
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("[%t] \"%r\" %s %b"))  // Custom format
            .service(
                SwaggerUi::new("/docs/{_:.*}")
                    .url("/docs/openapi.json", ApiDoc::openapi()),
            )
            .service(get_accounts_router())
            .app_data(web::Data::new(db.clone()))
    })
    .listen(listener).unwrap()
    .run();
    Ok(server)
}