.PHONY: help build run release clean test docker-build docker-run install-systemd

help:
	@echo "Gateway Service - Make commands"
	@echo ""
	@echo "Development:"
	@echo "  make build          - Build in debug mode"
	@echo "  make run            - Run in development mode"
	@echo "  make release        - Build optimized release binary"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build   - Build Docker image"
	@echo "  make docker-run     - Run in Docker container"
	@echo "  make docker-compose - Run all services with docker-compose"
	@echo ""
	@echo "Production:"
	@echo "  make install        - Install to /usr/local/bin"
	@echo "  make install-systemd - Install systemd service"
	@echo ""

build:
	cargo build

run:
	cargo run

release:
	cargo build --release

clean:
	cargo clean

test:
	curl -v http://localhost/auth/health || true
	curl -v http://localhost/booking/health || true
	curl -v http://localhost/mcp_service/health || true

docker-build:
	docker build -t gateway_service:latest .

docker-run:
	docker run -p 80:80 --env-file .env gateway_service:latest

docker-compose:
	docker-compose up --build

install: release
	sudo cp target/release/gateway_service /usr/local/bin/
	sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/gateway_service
	@echo "Gateway installed to /usr/local/bin/gateway_service"

install-systemd: install
	sudo mkdir -p /opt/gateway_service
	sudo cp .env /opt/gateway_service/
	sudo cp gateway.service /etc/systemd/system/
	sudo systemctl daemon-reload
	sudo systemctl enable gateway.service
	@echo "Systemd service installed. Start with: sudo systemctl start gateway"
