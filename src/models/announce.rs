use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Announcement {
    pub id: String,
    pub channel_id: String,
    pub message: String,
    pub frequency: String,
    pub last_posted: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnnouncementResponse {
    pub id: String,
    pub channel_id: String,
    pub message: String,
    pub frequency: String,
    pub last_posted: NaiveDateTime,
}

async fn query_list(pool: &PgPool, page: i64, size: i64) -> sqlx::Result<Vec<AnnouncementResponse>> {
    let rows = sqlx::query_as::<_, Announcement>(
        r#"SELECT id, channel_id, message, frequency, last_posted, created_at, updated_at
           FROM announcements
           ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
    )
    .bind(size)
    .bind(page * size)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Announcement::to_response).collect())
}

async fn query_find_by_id(pool: &PgPool, id: &str) -> sqlx::Result<Option<Announcement>> {
    sqlx::query_as::<_, Announcement>(
        r#"SELECT id, channel_id, message, frequency, last_posted, created_at, updated_at
           FROM announcements WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

async fn query_create(pool: &PgPool, channel_id: &str, message: &str, frequency: &str) -> sqlx::Result<Announcement> {
    let now = Utc::now().naive_utc();
    sqlx::query_as::<_, Announcement>(
        r#"INSERT INTO announcements (channel_id, message, frequency, last_posted, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, channel_id, message, frequency, last_posted, created_at, updated_at"#,
    )
    .bind(channel_id)
    .bind(message)
    .bind(frequency)
    .bind(now)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
}

async fn query_update(pool: &PgPool, id: &str, channel_id: Option<&str>, message: Option<&str>, frequency: Option<&str>) -> sqlx::Result<Option<AnnouncementResponse>> {
    let existing = query_find_by_id(pool, id).await?;
    let existing = match existing {
        Some(e) => e,
        None => return Ok(None),
    };

    let ch = channel_id.unwrap_or(&existing.channel_id);
    let msg = message.unwrap_or(&existing.message);
    let freq = frequency.unwrap_or(&existing.frequency);
    let now = Utc::now().naive_utc();

    let announcement = sqlx::query_as::<_, Announcement>(
        r#"UPDATE announcements
           SET channel_id = $1, message = $2, frequency = $3, updated_at = $4
           WHERE id = $5
           RETURNING id, channel_id, message, frequency, last_posted, created_at, updated_at"#,
    )
    .bind(ch)
    .bind(msg)
    .bind(freq)
    .bind(now)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(announcement.map(|a| a.to_response()))
}

async fn query_delete(pool: &PgPool, id: &str) -> sqlx::Result<u64> {
    let result = sqlx::query(
        r#"DELETE FROM announcements WHERE id = $1"#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

impl Announcement {
    pub fn to_response(self) -> AnnouncementResponse {
        AnnouncementResponse {
            id: self.id.clone(),
            channel_id: self.channel_id.clone(),
            message: self.message.clone(),
            frequency: self.frequency.clone(),
            last_posted: self.last_posted,
        }
    }

    pub async fn list_paginated(pool: &PgPool, page: i64, size: i64) -> sqlx::Result<Vec<AnnouncementResponse>> {
        query_list(pool, page, size).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &str) -> sqlx::Result<Option<Announcement>> {
        query_find_by_id(pool, id).await
    }

    pub async fn find_by_id_response(pool: &PgPool, id: &str) -> sqlx::Result<Option<AnnouncementResponse>> {
        query_find_by_id(pool, id).await.map(|opt| opt.map(|a| a.to_response()))
    }

    pub async fn create(pool: &PgPool, channel_id: &str, message: &str, frequency: &str) -> sqlx::Result<Announcement> {
        query_create(pool, channel_id, message, frequency).await
    }

    pub async fn update(pool: &PgPool, id: &str, channel_id: Option<&str>, message: Option<&str>, frequency: Option<&str>) -> sqlx::Result<Option<AnnouncementResponse>> {
        query_update(pool, id, channel_id, message, frequency).await
    }

    pub async fn delete(pool: &PgPool, id: &str) -> sqlx::Result<u64> {
        query_delete(pool, id).await
    }
}
