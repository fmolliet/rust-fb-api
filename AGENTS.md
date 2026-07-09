# fbrustapi - Agent Instructions

Single-crate Rust REST API: Axum 0.8 + SQLx 0.8 + PostgreSQL.

## Commands
```
DATABASE_URL=... cargo run          # start server on 0.0.0.0:8080
DATABASE_URL=... cargo test         # unit + integration tests
./run_local.sh dev                  # start DB (Docker/Colima) + migrations + server
./run_local.sh test                 # full stack: DB + migrations + cargo test + API health check
./run_local.sh reset                # drop+recreate public schema + reapply migrations
./run_local.sh clean                # remove fbrustapi-db container
```

DB container: `fbrustapi-db`, port 5432, user `postgres`, password `postgres`, db `fbrustapi`.

## Architecture
- `src/main.rs` — entrypoint. Creates PgPool, runs health check, mounts only `/health`. **Routes module (`src/routes/`) is empty** — not yet wired up. This is the only thing that prevents `/registration` and `/birthday` endpoints.
- `src/lib.rs` — declares modules: `config`, `db`, `error`, `models`, `dto`. Routes module is NOT declared yet.
- `src/config.rs` — reads `DATABASE_URL` env var. Exits on missing.
- `src/db.rs` — `PgPool` creation + health check query.
- `src/error.rs` — `AppError` enum (`NotFound`, `Conflict`, `BadRequest`, `Database`, `Internal`). Implements `IntoResponse` for Axum.
- `src/models/register.rs` — `Register` + `RegisterResponse` structs (no query methods yet).
- `src/models/birthday.rs` — `Birthday` + `BirthdayResponse` structs. GMT-3 timezone helper `today_day_month()`.
- `src/dto/register.rs` + `src/dto/birthday.rs` — serde `Deserialize` structs for request bodies.
- `migrations/` — raw SQL files piped to `psql` by `run_local.sh` (not sqlx migration tool).

## Key constraints
- `DATABASE_URL` is required at startup — no default, no `.env` file loading.
- Birthday dates use GMT-3 (`FixedOffset::west_opt(3 * 3600)`) for "today"/"month" endpoints.
- `registrations` table has partial unique index `uq_open_registration` on `(applicant_id) WHERE finished = FALSE` — prevents concurrent open registrations.
- `birthdays` table: `snowflake` is PK. `day` CHECK 1-31, `month` CHECK 1-12.
- SQLx features include both `postgres` AND `sqlite` — sqlite used for compile-time query validation but target DB is postgres only.
- Routes are planned but **not implemented**. `src/routes/` directory exists but is empty. Check `PLAN.md` for endpoint specs.

## Documentation
- `PLAN.md` — roadmap with task tracking (`[x]`/`[/]`/`[ ]`), endpoint contracts, DTOs, query API signatures. Keep updated.
- `okf/` — knowledge base with YAML frontmatter. Update `okf/index.md` when adding entries.
- `README.md` — tech stack, DB schema, API endpoints.
