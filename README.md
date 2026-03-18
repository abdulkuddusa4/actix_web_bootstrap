# actix-web-bootstrap

> ⚠️ **Work in Progress** — This project is actively being developed. Features and APIs may change.

A production-ready microservices bootstrap built with [Actix-Web](https://actix.rs/) in Rust. Designed to serve as a solid foundation for building scalable web backends with authentication, API gateway routing, payment processing, and third-party OAuth — all out of the box.

---

## Overview

`actix-web-bootstrap` is an opinionated starter template for microservices architecture in Rust. Instead of wiring up boilerplate every time you start a new project, this bootstrap gives you a running multi-service setup with the hard parts already handled: JWT-based auth, a reverse-proxy API gateway, Docker support, and integrations for Stripe and OAuth providers.

---

## Architecture

```
┌──────────────────────────────────┐
│         gateway_service          │  ← Single entry point for all clients
│     (Reverse Proxy / Router)     │
└───────────────┬──────────────────┘
                │
        ┌───────┴────────┐
        │                │
┌───────▼──────┐   ┌─────▼──────────┐
│ auth service │   │  other services │
│  (JWT, OAuth)│   │  (coming soon)  │
└──────────────┘   └────────────────┘
```

### Services

| Service | Description |
|---|---|
| `gateway_service` | API gateway — routes incoming requests to the appropriate downstream service |
| `auth` | Handles user registration, login, JWT issuance, and (planned) OAuth flows |

---

## Features

### ✅ Current
- Actix-Web microservices foundation
- API Gateway for centralized routing
- Auth service skeleton
- Dockerized services
- Makefile for common dev tasks

### 🔜 Planned
- **Stripe Payment Integration** — subscription and one-time payment flows via the Stripe API
- **Third-Party OAuth** — sign in with Google, GitHub, and other providers
- JWT authentication middleware
- Per-service environment configuration
- Docker Compose orchestration for local development

---

## Tech Stack

- **Language:** Rust
- **Web Framework:** [Actix-Web](https://actix.rs/)
- **Containerization:** Docker
- **Build Tooling:** Make, Shell scripts
- **Payments (planned):** Stripe
- **OAuth (planned):** Google, GitHub (and more)

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Docker](https://www.docker.com/)
- `make`

### Clone the repo

```bash
git clone https://github.com/abdulkuddusa4/actix_web_bootstrap.git
cd actix_web_bootstrap
```

### Run with Make

```bash
# Build all services
make build

# Run all services
make run
```

### Run with Docker

```bash
docker compose up --build
```

---

## Project Structure

```
actix_web_bootstrap/
├── auth/                   # Authentication service
├── gateway_service/        # API Gateway / reverse proxy
├── Makefile                # Dev task runner
└── README.md
```

---

## Roadmap

- [x] Project scaffold & microservice structure
- [x] API Gateway service
- [x] Auth service skeleton
- [ ] JWT middleware
- [ ] Chat System with group chat.
- [ ] Stripe payment integration
- [ ] Third-party OAuth (Google, GitHub)
- [ ] Docker Compose full orchestration
- [ ] CI/CD pipeline

---

## Contributing

This project is in early development. Contributions, issues, and feature requests are welcome once the core is more stable. Feel free to open an issue to start a discussion.

