---
type: BigQuery Table
title: Birthdays
description: One row per birthday record with Discord snowflake ID.
resource: migrations/0002_create_birthdays.sql
tags: [birthdays, snowflake, discord]
timestamp: 2026-08-27T00:00:00Z
---

# Schema

```sql
CREATE TABLE birthdays (
    day INT4 NOT NULL,
    month INT4 NOT NULL,
    name VARCHAR(255) NOT NULL,
    snowflake VARCHAR(255) NOT NULL,
    CONSTRAINT birthdays_pk PRIMARY KEY (snowflake),
    UNIQUE INDEX snowflake_un (snowflake)
);
```

# Model (Rust: `src/models/birthday.rs`)

```rust
pub struct Birthday {
    pub snowflake: String,
    pub name: String,
    pub day: i32,
    pub month: i32,
}

pub struct BirthdayResponse {
    pub snowflake: String,
    pub name: String,
    pub day: i32,
    pub month: i32,
}
```

# DTOs

## BirthdayRequestDto (`src/dto/birthday.rs`)

```rust
pub struct BirthdayRequestDto {
    #[validate(length(min = 1))]
    pub snowflake: String,
    #[validate(length(min = 1))]
    pub name: String,
    pub day: i32,
    pub month: i32,
}
```

# Operations (model methods — `src/models/birthday.rs`)

| Method | Signature |
|--------|-----------|
| create | `create(pool: &PgPool, dto: &BirthdayRequestDto) -> sqlx::Result<Birthday>` |
| find_by_snowflake | `find_by_snowflake(pool: &PgPool, snowflake: &str) -> sqlx::Result<Option<Birthday>>` |
| list_paginated | `list_paginated(pool: &PgPool, page: i64, size: i64) -> sqlx::Result<Vec<Birthday>>` |
| find_today | `find_today(pool: &PgPool) -> sqlx::Result<Vec<BirthdayResponse>>` |
| find_month | `find_month(pool: &PgPool) -> sqlx::Result<Vec<BirthdayResponse>>` |
| update | `update(pool: &PgPool, snowflake: &str, dto: &BirthdayRequestDto) -> sqlx::Result<Birthday>` |
| delete_by_snowflake | `delete_by_snowflake(pool: &PgPool, snowflake: &str) -> sqlx::Result<bool>` |

# Timezone

All "today" and "month" queries use `GMT-3` timezone (`FixedOffset::west_opt(3 * 3600)`). Implemented in `Birthday::today_day_month()` which converts UTC to GMT-3 before extracting day/month.

# Notes

- `snowflake` is Discord-style ID, stored as VARCHAR(255), used as PRIMARY KEY
- No `created_at` or `updated_at` columns — only `day`, `month`, `name`, `snowflake`
- `snowflake` must be unique across all records (primary key + unique index)
