# Migration Plan: Announce Service (Node.js/MongoDB → Rust Monolith/PostgreSQL)

## 1. Current Architecture

### Technology Stack
- **Runtime:** Node.js 18+ (TypeScript)
- **Framework:** Express.js
- **Database:** MongoDB (Mongoose ODM)
- **Logging:** Winston + Morgan
- **Container:** Docker (node:18-alpine, multi-stage)

### Service: Announce (Anuncio)
- CRUD for announcements sent to Facebook groups
- Single collection: `Anuncios` (4 fields)
- No authentication, no validation, no pagination

### Current Endpoints
| Method | Path | Description |
|--------|------|-------------|
| POST | `/anuncios` | Create announcement |
| GET | `/anuncios` | List all announcements |
| GET | `/anuncios/:id` | Get announcement by ID |
| PUT | `/anuncios/:id` | Update announcement |
| DELETE | `/anuncios/:id` | Delete announcement |

### Actuator Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/actuator/health` | Health check |
| GET | `/actuator/info` | App metadata |
| GET | `/actuator/app` | App info |

---

## 2. Target Architecture

### Technology Stack
- **Runtime:** Rust 1.80+ (stable)
- **Framework:** Axum (Tokio-based HTTP)
- **Database:** PostgreSQL 15+
- **ORM:** SQLx (compile-time verified queries) + Migration: diesel_cli or sqlx migrate
- **Async Runtime:** Tokio
- **Logging:** tracing + tracing-subscriber
- **Container:** Docker (rust:1.80-slim, multi-stage)
- **Validation:** serde + serde_json + custom validators

### Monolith Structure
```
announce-rs/
├── Cargo.toml
├── migrations/
│   ├── 20260716000000_create_anuncios.sql
│   └── ...
├── src/
│   ├── main.rs          # Entry point
│   ├── config.rs        # Environment config
│   ├── db.rs            # Database pool connection
│   ├── error.rs         # Unified error handling
│   ├── routes/
│   │   └── anuncios.rs  # CRUD routes
│   ├── models/
│   │   └── anuncio.rs   # SQLx models
│   ├── schemas/
│   │   └── anuncio.rs   # Request/response schemas
│   └── middleware/
│       ├── logging.rs   # Request logging
│       └── error.rs     # Error middleware
└── Dockerfile
```

---

## 3. Database Migration: MongoDB → PostgreSQL

### Current MongoDB Schema (Mongoose)
```
Anuncios Collection:
  message: String       (required)
  channelId: String     (required)
  frequency: String     (required)
  lastPosted: Date      (optional, default: now)
  _id: ObjectId         (auto-generated)
```

### Target PostgreSQL Schema
```sql
CREATE TABLE anuncios (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    message    TEXT         NOT NULL,
    channel_id VARCHAR(255) NOT NULL,
    frequency  VARCHAR(50)  NOT NULL,
    last_posted TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_anuncios_channel_id ON anuncios(channel_id);
CREATE INDEX idx_anuncios_last_posted ON anuncios(last_posted);
```

### Migration Strategy

#### Phase 1: Schema Mapping

| MongoDB Field | PostgreSQL Field | Type | Notes |
|---------------|------------------|------|-------|
| `_id` | `id` | `UUID` | Use `gen_random_uuid()` for generation |
| `message` | `message` | `TEXT` | Same semantics |
| `channelId` | `channel_id` | `VARCHAR(255)` | snake_case conversion |
| `frequency` | `frequency` | `VARCHAR(50)` | snake_case conversion |
| `lastPosted` | `last_posted` | `TIMESTAMPTZ` | ISO 8601 → TIMESTAMPTZ |

#### Phase 2: Data Migration

```sql
-- Step 1: Export from MongoDB (JSON)
-- mongodump --collection=anuncios --db=local --out=/tmp/mongo-export/

-- Step 2: Transform BSON to INSERT statements
-- Use a Node.js migration script with mongodb + pg drivers

-- Step 3: Insert into PostgreSQL
INSERT INTO anuncios (id, message, channel_id, frequency, last_posted)
VALUES (...);
```

Migration script (Node.js helper):
```typescript
// migrate.ts
import { MongoClient } from 'mongodb';
import { Pool } from 'pg';

async function migrate() {
  const mongo = new MongoClient(process.env.MONGO_URI);
  const pg = new Pool({ connectionString: process.env.PG_URI });

  await mongo.connect();
  const docs = await mongo.db('local').collection('anuncios').find({}).toArray();

  for (const doc of docs) {
    await pg.query(
      `INSERT INTO anuncios (id, message, channel_id, frequency, last_posted)
       VALUES ($1, $2, $3, $4, $5)
       ON CONFLICT (id) DO NOTHING`,
      [
        doc._id.toString(),
        doc.message,
        doc.channelId,
        doc.frequency,
        doc.lastPosted,
      ]
    );
  }

  await mongo.close();
  await pg.end();
}

migrate();
```

#### Phase 3: Cutover

1. Deploy Rust service alongside existing Node.js service (dual-write during transition)
2. Run data migration script during maintenance window
3. Switch traffic to Rust service
4. Verify data integrity
5. Decommission Node.js service and MongoDB

---

## 4. Rust Service: Detailed Specification

### 4.1 Routes & HTTP Methods

#### POST `/anuncios` — Create Announcement

**Request:**
```json
{
  "message": "Promoção de verão! 50% OFF",
  "channelId": "1234567890",
  "frequency": "diario"
}
```

**Response 201 Created:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Promoção de verão! 50% OFF",
  "channelId": "1234567890",
  "frequency": "diario",
  "lastPosted": "2026-07-16T10:30:00Z"
}
```

**Response 400 Bad Request:**
```json
{
  "error": "validation_error",
  "detail": "message is required"
}
```

**HTTP Status Codes:**
| Code | Condition |
|------|-----------|
| 201 | Created successfully |
| 400 | Invalid request body / validation error |
| 409 | Duplicate (if unique constraint added) |
| 500 | Internal server error |

---

#### GET `/anuncios` — List All Announcements

**Request:** No body. Optional query params:
- `limit` (int, default 100, max 1000)
- `offset` (int, default 0)

**Response 200 OK:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "message": "Promoção de verão! 50% OFF",
    "channelId": "1234567890",
    "frequency": "diario",
    "lastPosted": "2026-07-16T10:30:00Z"
  }
]
```

**HTTP Status Codes:**
| Code | Condition |
|------|-----------|
| 200 | Success |
| 400 | Invalid query params |
| 500 | Internal server error |

---

#### GET `/anuncios/{id}` — Get Announcement by ID

**Request:** Path param `id` (UUID string)

**Response 200 OK:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Promoção de verão! 50% OFF",
  "channelId": "1234567890",
  "frequency": "diario",
  "lastPosted": "2026-07-16T10:30:00Z"
}
```

**Response 404 Not Found:**
```json
{
  "error": "not_found",
  "detail": "Anúncio não encontrado"
}
```

**HTTP Status Codes:**
| Code | Condition |
|------|-----------|
| 200 | Found |
| 400 | Invalid UUID format |
| 404 | Announcement not found |
| 500 | Internal server error |

---

#### PUT `/anuncios/{id}` — Update Announcement

**Request:** Path param `id`, body with updateable fields (all optional):
```json
{
  "message": "Nova mensagem",
  "channelId": "0987654321",
  "frequency": "semanal"
}
```

**Response 200 OK:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Nova mensagem",
  "channelId": "0987654321",
  "frequency": "semanal",
  "lastPosted": "2026-07-16T11:00:00Z"
}
```

**Response 404 Not Found:**
```json
{
  "error": "not_found",
  "detail": "Anúncio não encontrado"
}
```

**HTTP Status Codes:**
| Code | Condition |
|------|-----------|
| 200 | Updated successfully |
| 400 | Invalid request body / UUID format |
| 404 | Not found |
| 500 | Internal server error |

---

#### DELETE `/anuncios/{id}` — Delete Announcement

**Request:** Path param `id` (UUID string)

**Response 200 OK:**
```json
{
  "message": "Anúncio deletado"
}
```

**Response 404 Not Found:**
```json
{
  "error": "not_found",
  "detail": "Anúncio não encontrado"
}
```

**HTTP Status Codes:**
| Code | Condition |
|------|-----------|
| 200 | Deleted successfully |
| 400 | Invalid UUID format |
| 404 | Not found |
| 500 | Internal server error |

---

#### GET `/actuator/health` — Health Check

**Response 200 OK:**
```json
{
  "status": "UP",
  "database": "connected"
}
```

**Response 503 Service Unavailable:**
```json
{
  "status": "DEGRADED",
  "database": "disconnected"
}
```

---

#### GET `/actuator/info` — App Info

**Response 200 OK:**
```json
{
  "name": "announce-service",
  "version": "2.0.0",
  "description": "Serviço de gerenciamento de anuncios do FB 2.0"
}
```

---

#### GET `/actuator/app` — App Details

**Response 200 OK:**
```json
{
  "name": "announce-service",
  "version": "2.0.0",
  "rust_version": "1.80.0",
  "environment": "production",
  "port": 3000
}
```

---

### 4.2 Request/Response Model

#### CreateAnuncioRequest
| Field | Type | Required | Max Length | Description |
|-------|------|----------|------------|-------------|
| message | string | Yes | 10000 | Announcement text content |
| channelId | string | Yes | 255 | Facebook group/channel ID |
| frequency | string | Yes | 50 | Posting frequency (diario/semanal/mensal) |

#### AnuncioResponse
| Field | Type | Description |
|-------|------|-------------|
| id | string (UUID) | Unique identifier |
| message | string | Announcement text |
| channelId | string | Facebook group/channel ID |
| frequency | string | Posting frequency |
| lastPosted | string (ISO 8601) | Last post timestamp |

#### UpdateAnuncioRequest
| Field | Type | Required | Max Length | Description |
|-------|------|----------|------------|-------------|
| message | string | No | 10000 | Announcement text content |
| channelId | string | No | 255 | Facebook group/channel ID |
| frequency | string | No | 50 | Posting frequency |

#### ErrorResponse
| Field | Type | Description |
|-------|------|-------------|
| error | string | Error type code |
| detail | string | Human-readable error message |

---

### 4.3 Rust Model Definition

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Anuncio {
    pub id: Uuid,
    pub message: String,
    pub channel_id: String,
    pub frequency: String,
    pub last_posted: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnuncioRequest {
    pub message: String,
    pub channel_id: String,
    pub frequency: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAnuncioRequest {
    pub message: Option<String>,
    pub channel_id: Option<String>,
    pub frequency: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub message: String,
}
```

---

### 4.4 Business Rules

| Rule | Validation | Error Response |
|------|------------|----------------|
| message is required | Not empty string | 400 `message is required` |
| channelId is required | Not empty string | 400 `channelId is required` |
| frequency is required | Not empty string | 400 `frequency is required` |
| message max length | ≤ 10000 chars | 400 `message exceeds maximum length` |
| channelId max length | ≤ 255 chars | 400 `channelId exceeds maximum length` |
| frequency max length | ≤ 50 chars | 400 `frequency exceeds maximum length` |
| id format (GET/PUT/DELETE) | Valid UUID | 400 `invalid id format` |
| id exists (GET/PUT/DELETE) | Record found in DB | 404 `Anúncio não encontrado` |

### 4.5 HTTP Method Matrix

| Route | POST | GET | PUT | DELETE |
|-------|:----:|:---:|:---:|:------:|
| `/anuncios` | ✅ 201 | ✅ 200 | ❌ | ❌ |
| `/anuncios/{id}` | ❌ | ✅ 200 | ✅ 200 | ✅ 200 |
| `/actuator/health` | ❌ | ✅ 200 | ❌ | ❌ |
| `/actuator/info` | ❌ | ✅ 200 | ❌ | ❌ |
| `/actuator/app` | ❌ | ✅ 200 | ❌ | ❌ |

### 4.6 Status Code Reference

| Code | Usage |
|------|-------|
| 200 | Success (GET, PUT, DELETE) |
| 201 | Created (POST) |
| 400 | Bad request (validation, invalid UUID) |
| 404 | Resource not found |
| 500 | Internal server error |
| 503 | Service unavailable (DB connection failed) |

---

## 5. Rust Project Dependencies (Cargo.toml)

```toml
[package]
name = "announce-service"
version = "2.0.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["json"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
dotenvy = "0.15"
thiserror = "1"
```

---

## 6. Migration Phases Summary

| Phase | Task | Duration |
|-------|------|----------|
| Phase 1 | Analyze current API, define Rust schemas | 1 day |
| Phase 2 | Set up Rust project structure, Cargo.toml | 1 day |
| Phase 3 | Write PostgreSQL migration SQL | 0.5 day |
| Phase 4 | Implement database models with SQLx | 1 day |
| Phase 5 | Implement CRUD routes (Axum) | 2 days |
| Phase 6 | Implement error handling, logging (tracing) | 1 day |
| Phase 7 | Implement actuator endpoints | 0.5 day |
| Phase 8 | Write data migration script | 1 day |
| Phase 9 | Docker container (multi-stage, rust:1.80-slim) | 0.5 day |
| Phase 10 | Integration tests | 2 days |
| Phase 11 | Staging deployment, data migration | 1 day |
| Phase 12 | Production cutover, monitoring | 1 day |

**Total Estimated: ~13 days**

---

## 7. Key Differences: MongoDB → PostgreSQL

| Aspect | MongoDB (Current) | PostgreSQL (Target) |
|--------|-------------------|---------------------|
| Schema | Dynamic, no validation | Strict, compile-time via SQLx |
| IDs | ObjectId (24-char hex) | UUID v4 |
| Nested objects | Supported natively | Requires JSONB or normalization |
| Queries | Mongoose find/update | SQLx prepared statements |
| Type safety | Runtime only | Compile-time verified |
| Connections | Driver pool | SQLx connection pool (bb8) |
| Migrations | None | sqlx migrate (versioned) |
| Null safety | Flexible | NOT NULL constraints |
| Transactions | Multi-doc (v4.0+) | Full ACID compliance |

---

## 8. Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `DATABASE_URL` | string | `postgres://localhost/announce` | PostgreSQL connection string |
| `PORT` | int | `3000` | HTTP listen port |
| `RUST_LOG` | string | `info` | Logging level (tracing) |
| `CORS_ORIGINS` | string | `*` | Comma-separated allowed origins |

---

## 9. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Data loss during migration | High | Backup MongoDB before migration, validate row counts |
| Breaking change: `_id` → `id` field rename | Medium | Keep response shape identical (`id` instead of `_id` may need adapter) |
| Breaking change: ObjectId → UUID | Medium | Store ObjectId as string in UUID field during migration |
| PostgreSQL downtime during cutover | High | Dual-read during transition period |
| Rust learning curve for team | Medium | Use well-established crates, start simple |

---

## 10. Response Format Note

The original API returns MongoDB driver objects directly (including `_id` field). The Rust implementation must:

1. Rename `_id` → `id` in all JSON responses
2. Return formatted error objects instead of raw error instances
3. Return 404 with `{ "message": "Anúncio não encontrado" }` — maintain compatibility
4. Return `503` for health check when DB is unavailable (was silently ignored before)
