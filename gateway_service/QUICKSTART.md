# Quick Start Guide

## Installation

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Place the gateway_service directory** at the same level as your other services:
   ```
   your-project/
   ├── backend_app/
   ├── booking_service/
   ├── inquary_mcp_server/
   └── gateway_service/         ← new directory
   ```

3. **Configure environment variables**:
   
   The `.env` file is already configured with your services:
   ```env
   OAUTH_SERVICE=http://127.0.0.1:3344
   BOOKING_SERVICE=http://127.0.0.1:1111
   MCP_SERVER=http://127.0.0.1:2222
   ```

## Running

### Option 1: Direct Run (Development)

```bash
cd gateway_service
cargo run
```

### Option 2: Using Make

```bash
cd gateway_service
make run
```

### Option 3: Release Build (Production)

```bash
cd gateway_service
make release
sudo ./target/release/gateway_service
```

Note: `sudo` is needed to bind to port 80. Alternatively, use capabilities:
```bash
sudo setcap 'cap_net_bind_service=+ep' ./target/release/gateway_service
./target/release/gateway_service
```

## Testing

Once the gateway is running:

```bash
# Run the test script
./test_gateway.sh

# Or test manually with curl
curl http://localhost/auth/some-endpoint
curl http://localhost/booking/api/slots
curl http://localhost/mcp_service/tools
```

## Running All Services Together

### Option 1: Docker Compose

```bash
cd gateway_service
docker-compose up --build
```

This will start all services (backend_app, booking_service, mcp_server, and gateway).

### Option 2: Manual (Run each in a separate terminal)

Terminal 1:
```bash
cd backend_app
# Run your backend app on port 3344
```

Terminal 2:
```bash
cd booking_service
# Run your booking service on port 1111
```

Terminal 3:
```bash
cd inquary_mcp_server
# Run your MCP server on port 2222
```

Terminal 4:
```bash
cd gateway_service
cargo run
```

## Verifying It Works

Your gateway should now be routing:
- `http://localhost/auth/*` → backend_app (port 3344)
- `http://localhost/booking/*` → booking_service (port 1111)
- `http://localhost/mcp_service/*` → inquary_mcp_server (port 2222)

Example:
- Request to: `http://localhost/auth/login`
- Gets routed to: `http://127.0.0.1:3344/login`

The prefix is automatically stripped before forwarding to the backend!

## Troubleshooting

### "Permission denied" when binding to port 80

Use one of these solutions:
1. Run with sudo: `sudo cargo run`
2. Use capabilities: `sudo setcap 'cap_net_bind_service=+ep' ./target/release/gateway_service`
3. Change port to 8080 (edit main.rs and change port from 80 to 8080)

### Backend services not responding

Make sure all three backend services are running:
```bash
netstat -tlnp | grep -E '3344|1111|2222'
```

### Environment variables not loading

Ensure `.env` is in the same directory where you run the command.

## Production Deployment

See the full README.md for:
- Systemd service setup
- Docker deployment
- Performance tuning
- Security considerations
