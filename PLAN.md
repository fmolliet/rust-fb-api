# Microserviço Rust REST API — Planejamento

**Status:** Em implementação  
**Status:** In Implementation  
**Stack:** Axum + SQLx (PostgreSQL) + Validator + Tracing  
**Plataforma:** ARM (native), Docker multi-stage build (multi-arch)

⸻

## Objetivo

Recriar endpoints Java (RESTEasy/Quarkus) em Rust com PostgreSQL.

* `/registration` — gerenciar processos de registro/apovação de usuários
* `/birthday` — gerenciar aniversários (snowflake = ID Discord)

Conexão DB via `DATABASE_URL` (env var).

⸻

## Stack

|Camada|Escolha|
|---|---|
|HTTP|axum|
|DB|sqlx (postgres, async, compile-time checked)|
|Validation|validator + serde|
|Logging|tracing + tracing-subscriber|
|UUID|uuid v7 |
|Dates|chrono (timezone GMT-3)|
|Config|serde + std::env|
|Errors|thiserror + anyhow|

⸻

## ✅ Atividades Concluídas

* [x] Cargo init + dependencies (Cargo.toml)
* [x] Migrations SQL (0001 + 0002)
* [x] Models (register.rs, birthday.rs)
* [x] DTOs (register.rs, birthday.rs)
* [x] Error handling (error.rs)
* [x] Config (config.rs) — load `DATABASE_URL`
* [x] DB connection (db.rs) — PgPool + health check
* [x] App entrypoint (src/main.rs) — tracing, pool, health check, axum::serve on 0.0.0.0:8080
* [x] Dockerized app starts and responds: `GET /health` → `{"status":"ok"}`
* [x] Fixed pg_hba.conf SSL/password auth issue (scram-sha-256 → trust)

⸻

## Estrutura (atualizada)

```
fbrustapi/
├── Cargo.toml                    # ✅
├── Dockerfile                    # ⏳
├── .dockerignore                 # ⏳
├── src/
│   ├── main.rs                   # ⏳
│   ├── config.rs                 # ⏳
│   ├── db.rs                     # ⏳
│   ├── error.rs                  # ✅
│   ├── models/
│   │   ├── register.rs           # ✅
│   │   └── birthday.rs           # ✅  (timezone GMT-3)
│   ├── dto/
│   │   ├── register.rs           # ✅
│   │   └── birthday.rs           # ✅
│   ├── routes/
│   │   ├── register.rs           # ⏳
│   │   └── birthday.rs           # ⏳
│   └── error.rs                  # ✅
└── migrations/
    ├── 0001_create_registrations.sql  # ✅
    └── 0002_create_birthdays.sql      # ✅
```

⸻

## Schema PostgreSQL

### registrations

```sql
CREATE TABLE registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    applicant_id TEXT NOT NULL,
    curator_id TEXT,
    birthday TEXT,
    specie TEXT,
    source TEXT,
    message_id TEXT
    finished BOOLEAN DEFAULT FALSE,
    approved BOOLEAN,    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE UNIQUE INDEX uq_open_registration
    ON registrations(applicant_id) WHERE finished = FALSE;
CREATE INDEX idx_reg_applicant ON registrations(applicant_id);
CREATE INDEX idx_reg_finished ON registrations(finished);
```

### birthdays

```sql
CREATE TABLE birthdays (
    snowflake TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    day INTEGER NOT NULL CHECK (day BETWEEN 1 AND 31),
    month INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

⸻

## DTOs

### CreateRegisterDto

```rust
#[derive(Debug, Deserialize)]
pub struct CreateRegisterDto {
    pub applicant_id: String,
    pub birthday: chrono::NaiveDate,
    pub specie: String,
    pub source: String,
    pub message_id: String,
}
```

### UpdateRegisterDto

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateRegisterDto {
    pub applicant_id: String,
    pub curator_id: String,
    pub approved: bool,
}
```

### BirthdayRequestDto

```rust
#[derive(Debug, Deserialize)]
pub struct BirthdayRequestDto {
    pub snowflake: String,
    pub name: String,
    pub day: u8,
    pub month: u8,
}
```

⸻

## Endpoints

### /registration

| Method | Route | DTO In | DTO Out | Status |
|---|---|---|---|---|
| POST | `/registration` | CreateRegisterDto | Register | 201 |
| GET | `/registration/open/{applicant_id}` | — | Register | 200/404 |
| GET | `/registration/{id}` | — | Register | 200/404 |
| PATCH | `/registration/{id}/finish` | UpdateRegisterDto | Register | 202 |
| DELETE | `/registration/{id}` | — | — | 204 |

#### Regras

1. `POST` — verifica registro aberto por `applicant_id`. Se existe → 400.
2. `PATCH` — busca registro aberto do applicant, seta `finished=true`, `curator_id`, `approved`.
3. UUID inválido → 400.
4. `DELETE` — remove por UUID. Se não existe → 404.

⸻

### /birthday

| Method | Route | DTO In | DTO Out | Status |
|---|---|---|---|---|
| GET | `/birthday?page=0&size=10` | — | BirthdayResponse [] | 200 |
| GET | `/birthday/today` | — | BirthdayResponse [] | 200 |
| GET | `/birthday/month` | — | BirthdayResponse [] | 200 |
| GET | `/birthday/{snowflake}` | — | BirthdayResponse | 200/404 |
| POST | `/birthday` | BirthdayRequestDto | Birthday | 201 |
| PATCH | `/birthday/{snowflake}` | BirthdayRequestDto | Birthday | 202 |
| DELETE | `/birthday/{snowflake}` | — | — | 204 |

#### Regras

1. `GET /birthday` — `LIMIT size OFFSET page * size`. Default size=10.
2. `GET /birthday/today` — WHERE `day = :d AND month = :m` (timezone GMT-3).
3. `GET /birthday/month` — WHERE `month = :m` (timezone GMT-3).
4. `POST` — verifica snowflake existente. Se existe → 400.
5. `snowflake` único por constraint DB.

⸻

## Query API (models)

### Register

```
find_unfinished_by_applicant_id(applicant_id: &str) -> Option<Register>
find_by_id(id: Uuid) -> Option<Register>
create(dto: &CreateRegisterDto) -> Register
finish(applicant_id: &str, curator_id: &str, approved: bool) -> Register
delete(id: Uuid) -> bool
```

### Birthday

```
find_by_snowflake(snowflake: &str) -> Option<Birthday>
list_paginated(page: i64, size: i64) -> Vec<Birthday>
find_today() -> Vec<Birthday>
find_month() -> Vec<Birthday>
create(dto: &BirthdayRequestDto) -> Birthday
update(snowflake: &str, dto: &BirthdayRequestDto) -> Birthday
delete(snowflake: &str) -> bool
```

⸻

## Config

Env var obrigatória:

```
DATABASE_URL=postgres://user:pass@host:5432/dbname
```

Sem → erro fatal ao iniciar.

⸻

## Dockerfile (multi-stage, multi-arch)

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

Build command (multi-arch):

```bash
docker buildx build --platform linux/arm64,linux/amd64 -t fbrustapi:latest .
```

⸻

## Melhorias sobre o original Java

1. UUID inválido → 400 Bad Request (validação no router)
2. Timestamps: `created_at`, `updated_at` em todas as tabelas
3. PATCH `/registration/{id}/finish` — mais explícito que body-only
4. GET `/registration/open/{applicant_id}` — rota clara, sem conflito com UUID
5. 404 explícito quando registro não existe (original retorna null)
6. Unique partial index protege concorrência no registro aberto
7. Tracing/structured logging ao invés de log.warn/log.info
8. Typed errors com `AppError` que mapeia para HTTP codes corretos
9. Timezone GMT-3 preservada (igual original Java)

⸻

## Próximos passos

1. [x] `cargo init + Cargo.toml com dependências
2. [x] `migrations/` com SQL schema
3. [x] `src/models/register.rs` — structs + queries
4. [x] `src/models/birthday.rs` — structs + queries
5. [x] `src/dto/register.rs` — serde structs
6. [x] `src/dto/birthday.rs` — serde structs
7. [x] `src/error.rs` — typed error response
8. [x] `src/config.rs` — load env
9. [x] `src/db.rs` — PgPool, connect, health
10. [x] `src/main.rs` — router, tracing, PgPool, server
11. [x] Dockerized app starts and responds
12. [ ] `src/routes/register.rs` — handlers registration (POST, GET open, GET id, PATCH finish, DELETE)
13. [ ] `src/routes/birthday.rs` — handlers birthday (CRUD, today, month)
14. [ ] Wire up routes in `main.rs` (extend router)
15. [ ] `Dockerfile` + `.dockerignore`
16. [ ] Build docker multi-arch
