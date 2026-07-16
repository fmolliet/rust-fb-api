use chrono::{Datelike, TimeZone};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::PgPool;
use crate::dto::birthday::BirthdayRequestDto;

const TZ: chrono::FixedOffset = chrono::FixedOffset::west_opt(3 * 3600).unwrap();

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Birthday {
    pub snowflake: String,
    pub name: String,
    pub day: i32,
    pub month: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BirthdayResponse {
    pub snowflake: String,
    pub name: String,
    pub day: i32,
    pub month: i32,
}

impl Birthday {
    pub fn today_day_month() -> (i32, i32) {
        let now = TZ.from_utc_datetime(&chrono::Utc::now().naive_utc());
        (now.day() as i32, now.month() as i32)
    }

    pub async fn create(pool: &PgPool, dto: &BirthdayRequestDto) -> sqlx::Result<Birthday> {
        sqlx::query_as::<_, Birthday>(
            r#"INSERT INTO birthdays (snowflake, name, day, month)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(&dto.snowflake)
        .bind(&dto.name)
        .bind(dto.day)
        .bind(dto.month)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_snowflake(pool: &PgPool, snowflake: &str) -> sqlx::Result<Option<Birthday>> {
        sqlx::query_as::<_, Birthday>(
            r#"SELECT * FROM birthdays WHERE snowflake = $1"#,
        )
        .bind(snowflake)
        .fetch_optional(pool)
        .await
    }

    pub async fn list_paginated(pool: &PgPool, page: i64, size: i64) -> sqlx::Result<Vec<Birthday>> {
        sqlx::query_as::<_, Birthday>(
            r#"SELECT * FROM birthdays ORDER BY snowflake DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(size)
        .bind(page * size)
        .fetch_all(pool)
        .await
    }

    pub async fn find_today(pool: &PgPool) -> sqlx::Result<Vec<BirthdayResponse>> {
        let (day, month) = Self::today_day_month();
        sqlx::query_as::<_, BirthdayResponse>(
            r#"SELECT snowflake, name, day, month FROM birthdays WHERE day = $1 AND month = $2"#,
        )
        .bind(day)
        .bind(month)
        .fetch_all(pool)
        .await
    }

    pub async fn find_month(pool: &PgPool) -> sqlx::Result<Vec<BirthdayResponse>> {
        let (_, month) = Self::today_day_month();
        sqlx::query_as::<_, BirthdayResponse>(
            r#"SELECT snowflake, name, day, month FROM birthdays WHERE month = $1"#,
        )
        .bind(month)
        .fetch_all(pool)
        .await
    }

    pub async fn update(pool: &PgPool, snowflake: &str, dto: &BirthdayRequestDto) -> sqlx::Result<Birthday> {
        sqlx::query_as::<_, Birthday>(
            r#"UPDATE birthdays
               SET name = $2, day = $3, month = $4
               WHERE snowflake = $1
               RETURNING *"#,
        )
        .bind(snowflake)
        .bind(&dto.name)
        .bind(dto.day)
        .bind(dto.month)
        .fetch_one(pool)
        .await
    }

    pub async fn delete_by_snowflake(pool: &PgPool, snowflake: &str) -> sqlx::Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM birthdays WHERE snowflake = $1"#,
        )
        .bind(snowflake)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

impl BirthdayResponse {
    pub fn today_day_month() -> (i32, i32) {
        let now = TZ.from_utc_datetime(&chrono::Utc::now().naive_utc());
        (now.day() as i32, now.month() as i32)
    }
}
