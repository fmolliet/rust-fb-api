---
type: BigQuery Table
title: Birthdays
description: One row per birthday record with Discord snowflake ID.
resource: migrations/0002_create_birthdays.sql
tags: [birthdays, snowflake, discord]
timestamp: 2026-07-02T00:00:00Z
---

# Schema

```sql
CREATE TABLE birthdays (
    snowflake TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    day INTEGER NOT NULL CHECK (day BETWEEN 1 AND 31),
    month INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

# Model (Rust: `src/models/birthday.rs`)

```rust
pub struct Birthday {
    pub snowflake: String,
    pub name: String,
    pub day: u8,
    pub month: u8,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

```rust
pub struct BirthdayResponse {
    pub snowflake: String,
    pub name: String,
    pub day: u8,
    pub month: u8,
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
    pub day: u8,
    pub month: u8,
}
```

# Operations (model methods — `src/models/birthday.rs`)

| Method | Signature |
|--------|-----------|
| find_by_snowflake | `find_by_snowflake(snowflake: &str) -> Option<Birthday>` |
| list_paginated | `list_paginated(page: i64, size: i64) -> Vec<Birthday>` |
| find_today | `find_today() -> Vec<Birthday>` |
| find_month | `find_month() -> Vec<Birthday>` |
| create | `create(dto: &BirthdayRequestDto) -> Birthday` |
| update | `update(snowflake: &str, dto: &BirthdayRequestDto) -> Birthday` |
| delete | `delete(snowflake: &str) -> bool` |

# Timezone

All "today" and "month" queries use `GMT-3` timezone (`FixedOffset::west_opt(3 * 3600)`). Implemented in `Birthday::today_day_month()` which converts UTC to GMT-3 before extracting day/month.

# Notes

- `snowflake` is Discord-style ID, stored as TEXT (not INT), used as PRIMARY KEY
- `day` constrained 1-31, `month` constrained 1-12 via CHECK constraints
- No `updated_at` column — only `created_at`
- `snowflake` must be unique across all records (primary key)
