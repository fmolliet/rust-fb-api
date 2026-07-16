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
* `/meeting` — gerenciar reuniões (CRUD completo, snowflake, soft delete, paginação)
* `/announce` — gerenciar mensagens de anúncio para servidor Discord

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
* [x] Routes module (src/routes/mod.rs)
* [x] Registration handlers (POST, GET open, GET id, PATCH finish, DELETE)
* [x] Birthday handlers (CRUD, today, month)
* [x] Wire up routes in main.rs
* [x] Fix SQLx type mismatches (u8→i16 for Postgres integers)
* [x] Fix AppState.pool extraction in route handlers
* [x] Dockerfile (multi-stage build)
* [x] .dockerignore

⸻

## Estrutura (atualizada)

```
fbrustapi/
├── Cargo.toml                    # ✅
├── Dockerfile                    # ✅
├── .dockerignore                 # ✅
├── src/
│   ├── main.rs                   # ✅
│   ├── config.rs                 # ✅
│   ├── db.rs                     # ✅
│   ├── error.rs                  # ✅
│   ├── lib.rs                    # ✅
│   ├── models/
│   │   ├── register.rs           # ✅
│   │   ├── birthday.rs           # ✅  (timezone GMT-3)
│   │   └── announce.rs           # ✅
│   ├── dto/
│   │   ├── register.rs           # ✅
│   │   ├── birthday.rs           # ✅
│   │   └── announce.rs           # ✅
│   ├── routes/
│   │   ├── mod.rs                # ✅
│   │   ├── register.rs           # ✅
│   │   ├── birthday.rs           # ✅
│   │   └── announce.rs           # ✅
│   └── error.rs                  # ✅
├── Dockerfile                    # ✅
├── .dockerignore                 # ✅
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

### meeting

```sql
CREATE TABLE meeting (
    id       BIGSERIAL PRIMARY KEY,
    name     VARCHAR(200) NOT NULL DEFAULT '',
    snowflake VARCHAR(50) NOT NULL UNIQUE,
    state    VARCHAR(2) NOT NULL,
    active   BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_meeting_state_active ON meeting(state, active);
CREATE INDEX idx_meeting_active ON meeting(active);
```

### DTOs — Meeting

#### CreateMeetingDto

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMeetingDto {
    #[validate(length(max = 200))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub snowflake: String,
    #[validate(length(equal = 2))]
    pub state: String,
}
```

#### UpdateMeetingDto

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMeetingDto {
    #[validate(length(max = 200))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub snowflake: String,
    #[validate(length(equal = 2))]
    pub state: String,
}
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

### announcements

```sql
CREATE TABLE announcements (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id TEXT         NOT NULL,
    message    TEXT         NOT NULL,
    frequency  VARCHAR(50)  NOT NULL DEFAULT 'daily',
    last_posted TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_announcements_channel_id ON announcements(channel_id);
CREATE INDEX idx_announcements_last_posted ON announcements(last_posted);
```

### DTOs — Announcement

#### CreateAnnouncementDto

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAnnouncementDto {
    #[validate(length(min = 1, max = 255))]
    pub channel_id: String,
    #[validate(length(max = 10000))]
    pub message: String,
    #[validate(length(max = 50))]
    pub frequency: Option<String>,
}
```

#### UpdateAnnouncementDto

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAnnouncementDto {
    #[validate(length(min = 1, max = 255))]
    pub channel_id: Option<String>,
    #[validate(length(max = 10000))]
    pub message: Option<String>,
    #[validate(length(max = 50))]
    pub frequency: Option<String>,
}
```

### Announcements

| Method | Route | DTO In | DTO Out | Status |
|---|---|---|---|---|
| GET | `/announce?page=0&size=100` | — | AnnouncementResponse [] | 200 |
| GET | `/announce/{id}` | — | AnnouncementResponse | 200/404 |
| POST | `/announce` | CreateAnnouncementDto | AnnouncementResponse | 201/400 |
| PATCH | `/announce/{id}` | UpdateAnnouncementDto | AnnouncementResponse | 200/400/404 |
| DELETE | `/announce/{id}` | — | — | 204/404 |

#### Regras

1. `POST /announce` — valida DTO campos obrigatórios. `frequency` padrão = "daily".
2. `GET /announce` — lista todas as announcements com paginação. Default size=100.
3. `GET /announce/{id}` — busca por UUID, retorna 404 se não encontrado.
4. `PATCH /announce/{id}` — partial update (todos os campos opcionais). Atualiza `updated_at`.
5. `DELETE /announce/{id}` — hard delete por UUID. 404 se não existe.
6. UUID inválido → 400.
7. `message` max 10000 chars, `channel_id` max 255 chars, `frequency` max 50 chars.

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

### /meeting

| Method | Route | DTO In | DTO Out | Status |
|---|---|---|---|---|
| GET | `/meeting?page=0&size=25` | — | MeetingResponse [] | 200 |
| GET | `/meeting/{snowflake}` | — | MeetingResponse | 200/404 |
| GET | `/meeting/state/{state}?page=0&size=25` | — | MeetingResponse [] | 200 |
| POST | `/meeting` | CreateMeetingDto | MeetingResponse | 201/400/409 |
| PUT | `/meeting` | UpdateMeetingDto | MeetingResponse | 200/400/404 |
| DELETE | `/meeting/{id}` | — | — | 204/404 |

#### Regras

1. `GET /meeting` — lista apenas reuniões ativas (`active = true`), paginação com default size=25
2. `GET /meeting/{snowflake}` — busca por snowflake (string), retorna 404 se não encontrado
3. `GET /meeting/state/{state}` — filtra por estado + ativo, paginação
4. `POST /meeting` — valida snowflake (1-50 chars) e state (exatamente 2 chars), unique constraint → 409
5. `PUT /meeting` — partial update (name opcional), reativa meeting (active=true), 404 se snowflake não existe
6. `DELETE /meeting/{id}` — soft delete via ID numérico, seta active=false

### /registration

| Method | Route | DTO In | DTO Out | Status |
|---|---|---|---|---|
| POST | `/registration` | CreateRegisterDto | Register | 201 |
| GET | `/registration/open/{applicant_id}` | — | Register | 200/404 |
| GET | `/registration/{id}` | — | Register | 200/404 |
| PATCH | `/registration/{id}` | UpdateRegisterDto | Register | 202 |
| DELETE | `/registration/{id}` | — | — | 204 |

#### Regras

1. `POST` — verifica registro aberto por `applicant_id`. Se existe → 400.
2. `PATCH /registration/{id}` — body contains `finished`, `curator_id`, `approved`. Searches for open registration by `applicant_id`, sets `finished=true`, `curator_id`, `approved`.
3. UUID inválido → 400.
4. `DELETE` — remove por UUID. Se não existe → 404.

⸻

## Notas — Sessão de Grilling (2026-07-16)

### Decisões de Arquitetura Validadas

| Decisão | Escolha | Motivo |
|---|---|---|
| HTTP Framework | Axum | Clean async ergonomics, tower integration, active maintenance |
| Database ORM | SQLx | Compile-time checked queries, no abstraction tax |
| Registrations IDs | UUID v7 | Sortable by creation time, avoids sequential collisions |
| Meetings IDs | BIGSERIAL | Paginated queries, no distributed ID need |
| Birthdays IDs | Snowflake | Discord compatibility (carry-over) |
| Validation | validator crate | Declarative, close to DTO definitions |
| Error Handling | thiserror + anyhow + AppError | Standard Rust web stack, predictable HTTP responses |
| Logging | tracing + tracing-subscriber | Structured logs, runtime-filterable |
| Config | std::env::var() | Single value (DATABASE_URL), minimal and explicit |
| Dates/Time | chrono | Battle-tested, good timezone support (GMT-3) |
| Migrations | SQL files + sqlx migrate | Simple, version-controlled, compile-time validated |
| Docker | cargo-chef multi-stage | CI/CD cache optimization |
| Config Mgmt | std::env | Single config value, no need for config crate |
| Pagination | Offset-based | Fine for current dataset sizes (<10k records) |

### Alterações Planejadas

1. **Endpoint simplificado**: `PATCH /registration/{id}` replaces `PATCH /registration/{id}/finish`. Body includes `finished: true`, `curator_id`, `approved`. Cleaner REST semantics, fewer endpoints.
2. **Meeting soft delete**: `active BOOLEAN` retained for historical records. Plan to purge old inactive meetings in data retention strategy.
3. **Future consideration**: Evaluate cursor-based pagination if tables exceed 10k records. Evaluate Liquibase if migration complexity increases.

### Stack Justification Summary

- **No regret on Axum** — default for new Rust web projects, best async story
- **No pushback on SQLx** — raw SQL with compile-time safety, no ORM overhead
- **Validator crate** — declarative rules, no cross-field validation needed
- **Chrono** — timezone support (GMT-3) working, no performance concerns
- **Tracing** — structured logging essential for production debugging

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

### Meeting

```
list_active(page: i64, size: i64) -> Vec<MeetingResponse>
find_by_snowflake(snowflake: &str) -> Option<Meeting>
find_by_snowflake_response(snowflake: &str) -> Option<MeetingResponse>
list_by_state(state: &str, page: i64, size: i64) -> Vec<MeetingResponse>
create(name: &str, snowflake: &str, state: &str) -> Meeting
update(name: &str, state: &str, snowflake: &str) -> MeetingResponse
soft_delete(id: i64) -> u64 (rows affected)
```

### Announcement

```
list_paginated(page: i64, size: i64) -> Vec<AnnouncementResponse>
find_by_id(id: Uuid) -> Option<Announcement>
create(channel_id: &str, message: &str, frequency: &str) -> Announcement
update(id: Uuid, channel_id: Option<&str>, message: Option<&str>, frequency: Option<&str>) -> Announcement
delete(id: Uuid) -> u64 (rows affected)
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
12. [x] `src/routes/register.rs` — handlers registration (POST, GET open, GET id, PATCH finish, DELETE)
13. [x] `src/routes/birthday.rs` — handlers birthday (CRUD, today, month)
14. [x] Wire up routes in `main.rs` (extend router)
15. [x] `Dockerfile` + `.dockerignore`
16. [x] Build docker multi-arch
17. [x] Integration tests
18. [x] `migrations/0003_create_meeting.sql` — meeting table + indexes
19. [x] `src/models/meeting.rs` — Meeting struct + 6 query methods
20. [x] `src/dto/meeting.rs` — CreateMeetingDto, UpdateMeetingDto (with validator)
21. [x] `src/routes/meeting.rs` — 6 route handlers + route registration
22. [x] Wire up `/meeting` routes in `main.rs`
23. [x] Unit tests for meeting DTOs (dto_tests.rs)
24. [x] Integration tests for meeting endpoint (15 tests covering all routes)
25. [x] `migrations/0004_create_announcements.sql` — announcements table + indexes
26. [x] `src/models/announce.rs` — Announcement struct + query methods
27. [x] `src/dto/announce.rs` — CreateAnnouncementDto, UpdateAnnouncementDto
28. [x] `src/routes/announce.rs` — route handlers + route registration
29. [x] Wire up `/announce` routes in `main.rs`
30. [x] Unit tests for announcement DTOs
31. [x] Integration tests for announce endpoint

⸻

## Próximos Passos (pós-grilling)

1. [ ] `src/routes/register.rs` — update PATCH handler: `/registration/{id}` instead of `/registration/{id}/finish`
2. [ ] `src/routes/register.rs` — update handler to accept `finished` field in UpdateRegisterDto
3. [ ] `tests/` — update integration tests for new registration PATCH endpoint path
4. [ ] `src/models/register.rs` — verify query method accepts `finished` flag in update
5. [ ] Data retention strategy — schedule job to purge inactive meetings older than X days
6. [ ] Consider Liquibase migration (long-term, if schema complexity grows)
7. [ ] Monitor pagination performance — switch to cursor-based if tables exceed 10k records
8. [ ] Add frequency-based filtering endpoint: `GET /announce/channel/{channel_id}?page=0&size=100`
9. [ ] Add actuator endpoints (`/actuator/info`, `/actuator/app`) for service metadata
10. [ ] Docker multi-stage build for announce service (if separate container)
