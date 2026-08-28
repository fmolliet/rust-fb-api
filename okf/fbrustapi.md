---
type: Service
title: fbrustapi
description: Microservice REST API built in Rust using Axum and SQLx with PostgreSQL backend.
resource: PLAN.md
tags: [service, rust, axum, sqlx, postgresql]
timestamp: 2026-08-27T00:00:00Z
---

# fbrustapi

Rust REST API microservice that replaces a Java (RESTEasy/Quarkus) implementation. Manages user registration, birthday tracking, meetings, and announcements via Discord snowflake IDs.

## Stack

| Layer | Choice |
|-------|--------|
| HTTP | `axum` 0.8 (macros, tokio, http1 features) |
| DB | `sqlx` 0.8 (postgres, async, compile-time checked queries) |
| Validation | `validator` 0.20 + serde |
| Logging | `tracing` + `tracing-subscriber` 0.3 (env-filter, json) |
| UUID | `uuid` 1 (v7, v4, serde) |
| Dates | `chrono` 0.4 (serde, std, GMT-3 timezone) |
| Config | `serde` + `std::env` |
| Errors | `thiserror` 2 + `anyhow` 1 |
| HTTP Middleware | `tower-http` 0.6 (trace) |
| Async Runtime | `tokio` 1 (full features) |
| Lints | `clippy` — `all` warn, `dbg_macro` deny, `todo` deny, `unused_imports` deny |

## Configuration

| Env Var | Required | Description |
|---------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string: `postgres://user:pass@host:5432/dbname` |
| `PORT` | No | Listen port (default: `8081`) |

Fails fast on startup if `DATABASE_URL` is not set.

## Docker

Multi-stage build (see `Dockerfile`):

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

`0.0.0.0:8081` (port from `PORT` env var, default 8081)

## Middleware

`x-correlation-id` — generates a UUID if not provided in the request header and echoes it back in the response.

## Structure

```
fbrustapi/
├── Cargo.toml
├── Dockerfile
├── .dockerignore
├── src/
│   ├── main.rs              — entrypoint, router, tracing, pool, server
│   ├── lib.rs               — public module exports
│   ├── config.rs            — load DATABASE_URL / PORT from env
│   ├── db.rs                — PgPool creation + health check + AppState
│   ├── error.rs             — AppError enum + IntoResponse
│   ├── middleware/
│   │   └── mod.rs           — correlation_id_middleware
│   ├── models/
│   │   ├── mod.rs           — re-exports
│   │   ├── register.rs      — Register model + queries
│   │   ├── birthday.rs      — Birthday / BirthdayResponse models
│   │   ├── meeting.rs       — Meeting / MeetingResponse models
│   │   └── announce.rs      — Announcement / AnnouncementResponse models
│   ├── dto/
│   │   ├── mod.rs           — re-exports
│   │   ├── register.rs      — CreateRegisterDto, UpdateRegisterDto
│   │   ├── birthday.rs      — BirthdayRequestDto
│   │   ├── meeting.rs       — CreateMeetingDto, UpdateMeetingDto
│   │   └── announce.rs      — CreateAnnouncementDto, UpdateAnnouncementDto
│   └── routes/
│       ├── mod.rs           — re-exports
│       ├── register.rs      — registration CRUD routes
│       ├── birthday.rs      — birthday CRUD routes
│       ├── meeting.rs       — meeting CRUD routes
│       └── announce.rs      — announcement CRUD routes
└── migrations/
    ├── 0001_create_registrations.sql
    ├── 0002_create_birthdays.sql
    ├── 0003_create_meeting.sql
    └── 0004_create_announcements.sql
```

## Status

| Component | Status |
|-----------|--------|
| Cargo.toml + deps | ✅ |
| Migrations (4) | ✅ |
| Models (register, birthday, meeting, announce) | ✅ |
| DTOs (register, birthday, meeting, announce) | ✅ |
| Error handling | ✅ |
| Config | ✅ |
| DB connection | ✅ |
| Health check | ✅ |
| Middleware (correlation-id) | ✅ |
| App entrypoint | ✅ |
| Docker | ✅ |
| Routes: registration | ✅ |
| Routes: birthday | ✅ |
| Routes: meeting | ✅ |
| Routes: announcement | ✅ |
| Clippy lints configured | ✅ |

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
10. Clippy lints — `all` warn, `dbg_macro`/`todo` deny
