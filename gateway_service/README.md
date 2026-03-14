# Gateway Service

A high-performance API gateway built with [Pingora](https://github.com/cloudflare/pingora) that routes requests to different backend services based on URL path prefixes.

## Features

- ⚡ High-performance reverse proxy using Cloudflare's Pingora
- 🔀 Path-based routing to multiple backend services
- 🔧 Configuration via environment variables
- 🚀 Low latency and high throughput
- 📝 Request path rewriting

## Architecture

```
Client Request
     ↓
Gateway (Port 80)
     ↓
  ┌──┴──────────────┬─────────────────┐
  ↓                 ↓                 ↓
/auth/*       /booking/*      /mcp_service/*
  ↓                 ↓                 ↓
OAUTH_SERVICE  BOOKING_SERVICE  MCP_SERVER
:3344             :1111            :2222
```

## Routing Rules

| Path Prefix      | Backend Service      | Example                                    |
|------------------|----------------------|--------------------------------------------|
| `/auth/*`        | OAUTH_SERVICE        | `/auth/login` → `http://127.0.0.1:3344/login` |
| `/booking/*`     | BOOKING_SERVICE      | `/booking/api/slots` → `http://127.0.0.1:1111/api/slots` |
| `/mcp_service/*` | MCP_SERVER           | `/mcp_service/tools` → `http://127.0.0.1:2222/tools` |

## Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- Running backend services on their respective ports

## Installation

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Navigate to the gateway directory**:
   ```bash
   cd gateway_service
   ```

3. **Configure environment variables**:
   
   Edit the `.env` file with your backend service URLs:
   ```env
   OAUTH_SERVICE=http://127.0.0.1:3344
   BOOKING_SERVICE=http://127.0.0.1:1111
   MCP_SERVER=http://127.0.0.1:2222
   ```

## Running the Gateway

### Development Mode

```bash
cargo run
```

### Production Mode (Optimized Build)

```bash
cargo build --release
./target/release/gateway_service
```

### Running with Logging

```bash
RUST_LOG=info cargo run
```

## Testing

Once the gateway is running, you can test it with curl:

```bash
# Test OAUTH_SERVICE routing
curl http://localhost/auth/some-endpoint

# Test BOOKING_SERVICE routing
curl http://localhost/booking/api/slots

# Test MCP_SERVER routing
curl http://localhost/mcp_service/tools
```

## Configuration

All configuration is done via the `.env` file:

| Variable          | Description                      | Default                  |
|-------------------|----------------------------------|--------------------------|
| `OAUTH_SERVICE`   | OAuth service backend URL        | http://127.0.0.1:3344   |
| `BOOKING_SERVICE` | Booking service backend URL      | http://127.0.0.1:1111   |
| `MCP_SERVER`      | MCP server backend URL           | http://127.0.0.1:2222   |

## Project Structure

```
gateway_service/
├── Cargo.toml          # Rust dependencies
├── src/
│   └── main.rs         # Gateway implementation
├── .env                # Environment configuration
├── .gitignore          # Git ignore rules
└── README.md           # This file
```

## How It Works

1. **Request Reception**: The gateway listens on port 80 for incoming HTTP requests
2. **Path Matching**: It examines the request path and matches it against configured prefixes
3. **Backend Selection**: Based on the prefix, it selects the appropriate backend service
4. **Path Rewriting**: The prefix is stripped from the path before forwarding
5. **Request Forwarding**: The modified request is sent to the selected backend
6. **Response Relay**: The backend's response is relayed back to the client

## Running in Production

For production deployment, consider:

1. **Run as a systemd service**:
   ```bash
   sudo cp target/release/gateway_service /usr/local/bin/
   # Create a systemd service file
   ```

2. **Use a process manager** like `systemd` or `supervisor`

3. **Set appropriate permissions** for port 80:
   ```bash
   sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/gateway_service
   ```

4. **Configure logging**:
   ```bash
   RUST_LOG=info,gateway_service=debug
   ```

## Docker Support

Create a `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gateway_service /usr/local/bin/
COPY .env /app/.env
WORKDIR /app
EXPOSE 80
CMD ["gateway_service"]
```

Build and run:
```bash
docker build -t gateway_service .
docker run -p 80:80 --env-file .env gateway_service
```

## Performance

Pingora is used by Cloudflare to handle millions of requests per second. This gateway inherits:
- Async I/O for high concurrency
- Low memory footprint
- Efficient connection pooling
- HTTP/1.1 and HTTP/2 support

## Troubleshooting

### Port 80 Permission Denied

On Linux, binding to port 80 requires root privileges. Options:

1. Run with sudo: `sudo cargo run`
2. Use capabilities: `sudo setcap 'cap_net_bind_service=+ep' ./target/release/gateway_service`
3. Change to a higher port (e.g., 8080) and use nginx/iptables for port forwarding

### Backend Connection Refused

Ensure all backend services are running:
```bash
# Check if services are listening
netstat -tlnp | grep -E '3344|1111|2222'
```

### Environment Variables Not Loading

Make sure the `.env` file is in the same directory where you run the command, or use absolute paths.

## License

MIT

## Contributing

Contributions welcome! Please open an issue or submit a pull request.
