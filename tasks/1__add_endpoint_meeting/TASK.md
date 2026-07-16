# Task: Add Meeting Endpoint

## Summary

Implement the full meeting CRUD microservice endpoint following the existing codebase patterns (axum + sqlx + validator + thiserror).

Base: `tasks/1__add_endpoint_meeting/PLAN.md` and `tasks/1__add_endpoint_meeting/openapi.yml`

## Schema

```sql
CREATE TABLE IF NOT EXISTS meeting (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(200) NOT NULL DEFAULT '',
    snowflake   VARCHAR(50) NOT NULL UNIQUE,
    state       VARCHAR(2) NOT NULL,
    active      BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_meeting_state_active ON meeting(state, active);
CREATE INDEX IF NOT EXISTS idx_meeting_active ON meeting(active);
```

Migration file: `migrations/0003_create_meeting.sql`

## Files to Create

1. `src/models/meeting.rs` — Meeting struct + query methods
2. `src/dto/meeting.rs` — DTOs (MeetingResponse, CreateMeetingDto, UpdateMeetingDto)
3. `src/routes/meeting.rs` — Route handlers + route registration
4. `migrations/0003_create_meeting.sql` — DB migration

## Files to Modify

1. `src/models/mod.rs` — add `pub mod meeting;`
2. `src/dto/mod.rs` — add `pub mod meeting;`
3. `src/routes/mod.rs` — add `pub mod meeting;`
4. `src/main.rs` — wire up `/meeting` routes

## DTOs

### MeetingResponse

```rust
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MeetingResponse {
    pub id: i64,
    pub name: String,
    pub snowflake: String,
    pub state: String,
}
```

### CreateMeetingDto

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

### UpdateMeetingDto

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

## Route Handlers

### `GET /meeting` — List active meetings (paginated)

- Query params: `page` (i64, default 0), `size` (i64, default 25)
- Returns: `[MeetingResponse]` (only `active = true`)
- Status: 200

### `GET /meeting/{snowflake}` — Get by snowflake

- Path param: `snowflake` (String)
- Returns: `MeetingResponse`
- Status: 200 / 404

### `GET /meeting/state/{state}` — List by state (paginated)

- Path param: `state` (String, 2 chars)
- Query params: `page`, `size`
- Returns: `[MeetingResponse]` (only `active = true` + matching state)
- Status: 200

### `POST /meeting` — Create meeting

- Body: `CreateMeetingDto`
- Inserts with `active = true`
- Status: 201 Created
- Conflict if snowflake exists → 409

### `PUT /meeting` — Update meeting (partial update, reactivates)

- Body: `UpdateMeetingDto`
- Updates `name` and `state` if non-empty
- Always sets `active = true`
- Status: 200 OK
- Not found if snowflake doesn't exist → 404

### `DELETE /meeting/{id}` — Soft delete by numeric ID

- Path param: `id` (i64)
- Sets `active = false`
- Status: 200 OK
- Not found if ID doesn't exist → 404

## Model Methods

```rust
impl Meeting {
    pub async fn list_active(pool: &PgPool, page: i64, size: i64) -> sqlx::Result<Vec<Meeting>>
    pub async fn find_by_snowflake(pool: &PgPool, snowflake: &str) -> sqlx::Result<Option<Meeting>>
    pub async fn list_by_state(pool: &PgPool, state: &str, page: i64, size: i64) -> sqlx::Result<Vec<Meeting>>
    pub async fn create(pool: &PgPool, dto: &CreateMeetingDto) -> sqlx::Result<Meeting>
    pub async fn update(pool: &PgPool, dto: &UpdateMeetingDto) -> sqlx::Result<Meeting>
    pub async fn soft_delete(pool: &PgPool, id: i64) -> sqlx::Result<bool>
}
```

## Important Notes

- Follow existing patterns from `birthday.rs` and `register.rs` for structure
- Use `AppError` from `error.rs` (already has NotFound, Conflict, BadRequest)
- Default page size is 25 (not 1000 like the original Java — improvement #4 from PLAN.md)
- Return 404 on not found (unlike the original Java which returns null — improvement #5 from PLAN.md)
- Return 404 on update when snowflake doesn't exist (fixes NPE bug in original)
- Use `serde_json::json!` for any JSON responses
- Validation errors from `validator` crate: validate in handler, return 400 with field details
- Use `sqlx::query_as!` or `sqlx::query_as::<_, _>` for typed queries
- `snowflake` unique constraint violation returns SQL state `23505` → map to 409 Conflict
