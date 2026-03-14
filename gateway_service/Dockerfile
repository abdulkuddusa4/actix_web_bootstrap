FROM rust:1.75-bookworm as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/gateway_service /usr/local/bin/gateway_service

# Copy .env file (optional, can be overridden with docker run --env-file)
COPY .env /app/.env

WORKDIR /app

# Expose port 80
EXPOSE 80

# Run the gateway
CMD ["gateway_service"]
