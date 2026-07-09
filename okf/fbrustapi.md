---
type: Service
title: fbrustapi
description: Microservice REST API built in Rust using Axum and SQLx with PostgreSQL backend.
resource: PLAN.md
tags: [service, rust, axum, sqlx, postgresql]
timestamp: 2026-07-02T00:00:00Z
---

# fbrustapi

Rust REST API microservice that replaces a Java (RESTEasy/Quarkus) implementation. Manages user registration processes and birthday tracking via Discord snowflake IDs.

## Stack

| Layer | Choice |
|-------|--------|
| HTTP | `axum` 0.8 (macros, tokio, http1 features) |
| DB | `sqlx` 0.8 (postgres, sqlite, async, compile-time checked queries) |
| Validation | `validator` 0.20 + serde |
| Logging | `tracing` + `tracing-subscriber` 0.3 (env-filter) |
| UUID | `uuid` 1 (v7 + serde) |
| Dates | `chrono` 0.4 (serde, GMT-3 timezone) |
| Config | `serde` + `std::env` |
| Errors | `thiserror` 2 + `anyhow` 1 |
| HTTP Middleware | `tower-http` 0.6 (trace) |
| Async Runtime | `tokio` 1 (full features) |

## Configuration

| Env Var | Required | Description |
|---------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string: `postgres://user:pass@host:5432/dbname` |

Fails fast on startup if `DATABASE_URL` is not set.

## Docker

Multi-stage, multi-arch build:

```dockerfile
FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin app

FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/app /usr/local/bin/app
EXPOSE 8080
CMD ["app"]
```

Build command:
```bash
docker buildx build --platform linux/arm64,linux/amd64 -t fbrustapi:latest .
```

## Listening

`0.0.0.0:8080`

## Structura

```
fbrustapi/
├── Cargo.toml
├── Dockerfile
├── .dockerignore
├── src/
│   ├── main.rs              — entrypoint, router, tracing, pool, server
│   ├── config.rs            — load DATABASE_URL from env
│   ├── db.rs                — PgPool creation + health check
│   ├── error.rs             — AppError enum + IntoResponse
│   ├── models/
│   │   ├── mod.rs           — re-exports
│   │   ├── register.rs      — Register / RegisterResponse structs
│   │   └── birthday.rs      — Birthday / BirthdayResponse structs + GMT-3 helper
│   ├── dto/
│   │   ├── register.rs      — CreateRegisterDto, UpdateRegisterDto
│   │   └── birthday.rs      — BirthdayRequestDto
│   └── routes/
│       ├── register.rs      — ⏳ not yet implemented
│       └── birthday.rs      — ⏳ not yet implemented
└── migrations/
    ├── 0001_create_registrations.sql
    └── 0002_create_birthdays.sql
```

## Status

| Component | Status |
|-----------|--------|
| Cargo.toml + deps | ✅ |
| Migrations | ✅ |
| Models (register, birthday) | ✅ |
| DTOs (register, birthday) | ✅ |
| Error handling | ✅ |
| Config | ✅ |
| DB connection | ✅ |
| Health check | ✅ |
| App entrypoint | ✅ |
| Docker | ⏳ |
| Routes: registration | ⏳ |
| Routes: birthday | ⏳ |
| Wire up routes in main.rs | ⏳ |
| .dockerignore | ⏳ |

## Improvements over Java (RESTEasy/Quarkus)

1. Invalid UUID → 400 (validation at router level)
2. `created_at` + `updated_at` timestamps on all tables
3. Explicit `PATCH /registration/{id}/finish` instead of body-only
4. `GET /registration/open/{applicant_id}` — clear route, no UUID conflict
5. 404 when registration not found (original returns null)
6. Partial unique index guards against concurrent open registrations
7. Structured `tracing` logging vs log.warn/log.info
8. Typed `AppError` mapping to correct HTTP codes
9. GMT-3 timezone preserved (same as Java original)
