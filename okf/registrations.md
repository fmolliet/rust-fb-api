---
type: BigQuery Table
title: Registrations
description: One row per user registration/approval process in the fbrustapi system.
resource: migrations/0001_create_registrations.sql
tags: [registrations, users, approval]
timestamp: 2026-08-27T00:00:00Z
---

# Schema

```sql
CREATE TABLE registrations (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    applicantid VARCHAR(255) NOT NULL,
    curatorid VARCHAR(255) NULL,
    finished BOOL NULL DEFAULT false,
    messageid VARCHAR(255) NULL,
    birthday VARCHAR(255) NULL,
    specie VARCHAR(255) NULL,
    source VARCHAR(255) NULL,
    createdat TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updatedat TIMESTAMP NOT NULL DEFAULT current_timestamp,
    approved BOOL NULL,
    CONSTRAINT registrations_pkey PRIMARY KEY (id),
    INDEX idx_applicantid (applicantid)
);
```

# Model (Rust: `src/models/register.rs`)

```rust
pub struct Register {
    pub id: Uuid,
    #[sqlx(rename = "applicantid")]
    pub applicant_id: String,
    #[sqlx(rename = "curatorid")]
    pub curator_id: Option<String>,
    pub birthday: Option<String>,
    pub specie: Option<String>,
    pub source: Option<String>,
    #[sqlx(rename = "messageid")]
    pub message_id: String,
    pub finished: Option<bool>,
    pub approved: Option<bool>,
    #[sqlx(rename = "createdat")]
    pub created_at: chrono::NaiveDateTime,
    #[sqlx(rename = "updatedat")]
    pub updated_at: chrono::NaiveDateTime,
}
```

# DTOs

## CreateRegisterDto (`src/dto/register.rs`)

```rust
pub struct CreateRegisterDto {
    #[validate(length(min = 1))]
    pub applicant_id: Option<String>,
    pub birthday: Option<String>,
    #[validate(length(min = 1))]
    pub specie: Option<String>,
    #[validate(length(min = 1))]
    pub source: Option<String>,
    #[validate(length(min = 1))]
    pub message_id: Option<String>,
}
```

## UpdateRegisterDto (`src/dto/register.rs`)

```rust
pub struct UpdateRegisterDto {
    #[validate(length(min = 1))]
    pub applicant_id: String,
    #[validate(length(min = 1))]
    pub curator_id: String,
    pub approved: bool,
}
```

# Operations (model methods — `src/models/register.rs`)

| Method | Signature |
|--------|-----------|
| create | `create(pool: &PgPool, dto: &CreateRegisterDto) -> sqlx::Result<Register>` |
| find_unfinished_by_applicant_id | `find_unfinished_by_applicant_id(pool: &PgPool, applicant_id: &str) -> sqlx::Result<Option<Register>>` |
| find_by_id | `find_by_id(pool: &PgPool, id: &str) -> sqlx::Result<Option<Register>>` |
| finish | `finish(pool: &PgPool, applicant_id: &str, dto: &UpdateRegisterDto) -> sqlx::Result<Register>` |
| delete_by_id | `delete_by_id(pool: &PgPool, id: &str) -> sqlx::Result<bool>` |

# Notes

- `finished = FALSE` means an open/pending registration
- `approved` is nullable — not yet decided vs explicitly approved/rejected
- Timestamps stored as `TIMESTAMP`, accessed via `chrono::NaiveDateTime`
- Column names in DB use no underscore (`applicantid`, `curatorid`, etc.), mapped via `#[sqlx(rename = ...)]`
