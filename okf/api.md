---
type: API
title: REST API Endpoints
description: All HTTP endpoints for the fbrustapi service.
resource: PLAN.md
tags: [api, http, rest, endpoints]
timestamp: 2026-07-02T00:00:00Z
---

# Endpoints

## /registration

| Method | Route | Request Body | Response | Status |
|--------|-------|-------------|----------|--------|
| POST | `/registration` | CreateRegisterDto | Register | 201 |
| GET | `/registration/open/{applicant_id}` | — | Register | 200/404 |
| GET | `/registration/{id}` | — | Register | 200/404 |
| PATCH | `/registration/{id}/finish` | UpdateRegisterDto | Register | 202 |
| DELETE | `/registration/{id}` | — | — | 204 |

### Rules

1. `POST` — checks for existing open registration by `applicant_id`. If exists → 400.
2. `PATCH` — lookup open registration by applicant_id, set `finished=true`, `curator_id` and `approved`.
3. Invalid UUID → 400 Bad Request.
4. `DELETE` — remove by UUID. If not found → 404.

## /birthday

| Method | Route | Query Params | Request Body | Response | Status |
|--------|-------|-------------|-------------|----------|--------|
| GET | `/birthday` | `page=0&size=10` | — | BirthdayResponse[] | 200 |
| GET | `/birthday/today` | — | — | BirthdayResponse[] | 200 |
| GET | `/birthday/month` | `month=N` | — | BirthdayResponse[] | 200 |
| GET | `/birthday/{snowflake}` | — | — | BirthdayResponse | 200/404 |
| POST | `/birthday` | — | BirthdayRequestDto | Birthday | 201 |
| PATCH | `/birthday/{snowflake}` | — | BirthdayRequestDto | Birthday | 202 |
| DELETE | `/birthday/{snowflake}` | — | — | — | 204 |

### Rules

1. `GET /birthday` — `LIMIT size OFFSET page * size`. Default size=10.
2. `GET /birthday/today` — WHERE `day = :d AND month = :m` (timezone GMT-3).
3. `GET /birthday/month` — WHERE `month = :m` (timezone GMT-3).
4. `POST` — checks for existing snowflake. If exists → 400.
5. `snowflake` is unique via DB primary key constraint.

## /health

| Method | Route | Response | Status |
|--------|-------|----------|--------|
| GET | `/health` | `"ok"` | 200 |

Simple health check endpoint, no request body, returns string.

# Error Format

All errors return JSON:

```json
{
  "error": "message"
}
```

| Status | Code | Usage |
|--------|------|-------|
| 400 | BadRequest | Invalid input, duplicate registration, bad UUID |
| 404 | NotFound | Resource not found |
| 409 | Conflict | Conflict (e.g. duplicate snowflake) |
| 500 | Internal / Database | Unexpected server or DB error |

# AppError Variants

- `NotFound(String)` → 404
- `Conflict(String)` → 409
- `BadRequest(String)` → 400
- `Database(sqlx::Error)` → 500 (logs error internally)
- `Internal(anyhow::Error)` → 500 (logs error internally)

# Notes

- Routes for `/registration` and `/birthday` handlers are planned but not yet implemented (marked `⏳` in PLAN.md)
- Currently only `/health` is live
- Router setup is in `src/main.rs` using `axum::Router`
- Currently running on `0.0.0.0:8080`
