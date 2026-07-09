---
type: BigQuery Table
title: Registrations
description: One row per user registration/approval process in the fbrustapi system.
resource: migrations/0001_create_registrations.sql
tags: [registrations, users, approval]
timestamp: 2026-07-02T00:00:00Z
---

# Schema

```sql
CREATE TABLE registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    applicant_id TEXT NOT NULL,
    curator_id TEXT,
    birthday TEXT,
    specie TEXT,
    source TEXT,
    message_id TEXT,
    finished BOOLEAN DEFAULT FALSE,
    approved BOOLEAN,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

# Indexes

| Index | Type | Column(s) | Condition |
|-------|------|-----------|-----------|
| uq_open_registration | UNIQUE | applicant_id | WHERE finished = FALSE |
| idx_reg_applicant | BTREE | applicant_id | — |
| idx_reg_finished | BTREE | finished | — |

# Model (Rust: `src/models/register.rs`)

```rust
pub struct Register {
    pub id: Uuid,
    pub applicant_id: String,
    pub curator_id: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub specie: String,
    pub source: String,
    pub message_id: String,
    pub finished: bool,
    pub approved: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

```rust
pub struct RegisterResponse {
    pub id: Uuid,
    pub applicant_id: String,
    pub curator_id: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub specie: String,
    pub source: String,
    pub message_id: String,
    pub finished: bool,
    pub approved: Option<bool>,
}
```

# DTOs

## CreateRegisterDto (`src/dto/register.rs`)

```rust
pub struct CreateRegisterDto {
    #[validate(length(min = 1))]
    pub applicant_id: String,
    pub birthday: chrono::NaiveDate,
    #[validate(length(min = 1))]
    pub specie: String,
    #[validate(length(min = 1))]
    pub source: String,
    #[validate(length(min = 1))]
    pub message_id: String,
}
```

## UpdateRegisterDto (`src/dto/register.rs`)

```rust
pub struct UpdateRegisterDto {
    #[validate(length(min = 1))]
    pub applicant_id: String,
    pub curator_id: String,
    pub approved: bool,
}
```

# Operations (model methods — `src/models/register.rs`)

| Method | Signature |
|--------|-----------|
| find_unfinished_by_applicant_id | `find_unfinished_by_applicant_id(applicant_id: &str) -> Option<Register>` |
| find_by_id | `find_by_id(id: Uuid) -> Option<Register>` |
| create | `create(dto: &CreateRegisterDto) -> Register` |
| finish | `finish(applicant_id: &str, curator_id: &str, approved: bool) -> Register` |
| delete | `delete(id: Uuid) -> bool` |

# Notes

- `uq_open_registration` partial index protects against duplicate open registrations (concurrency guard)
- `finished = FALSE` means an open/pending registration
- `approved` is nullable — not yet decided vs explicitly approved/rejected
- Timestamps stored as `TIMESTAMPTZ`, converted to UTC in Rust via `chrono::DateTime<Utc>`
