use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Meeting {
    pub id: i64,
    pub name: String,
    pub snowflake: String,
    pub state: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MeetingResponse {
    pub id: i64,
    pub name: String,
    pub snowflake: String,
    pub state: String,
}

impl Meeting {
    pub fn to_response(self) -> MeetingResponse {
        MeetingResponse {
            id: self.id,
            name: self.name,
            snowflake: self.snowflake,
            state: self.state,
        }
    }

    pub async fn list_active(pool: &PgPool, page: i64, size: i64) -> sqlx::Result<Vec<MeetingResponse>> {
        let rows = sqlx::query_as::<_, Meeting>(
            r#"SELECT id, name, snowflake, state, active FROM meeting
               WHERE active = $1 ORDER BY id DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(true)
        .bind(size)
        .bind(page * size)
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(Meeting::to_response).collect())
    }

    pub async fn find_by_snowflake(pool: &PgPool, snowflake: &str) -> sqlx::Result<Option<Meeting>> {
        sqlx::query_as::<_, Meeting>(
            r#"SELECT id, name, snowflake, state, active FROM meeting WHERE snowflake = $1"#,
        )
        .bind(snowflake)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_snowflake_response(pool: &PgPool, snowflake: &str) -> sqlx::Result<Option<MeetingResponse>> {
        let opt = Self::find_by_snowflake(pool, snowflake).await?;
        Ok(opt.map(Meeting::to_response))
    }

    pub async fn list_by_state(pool: &PgPool, state: &str, page: i64, size: i64) -> sqlx::Result<Vec<MeetingResponse>> {
        let rows = sqlx::query_as::<_, Meeting>(
            r#"SELECT id, name, snowflake, state, active FROM meeting
               WHERE active = $1 AND state = $2 ORDER BY id DESC LIMIT $3 OFFSET $4"#,
        )
        .bind(true)
        .bind(state)
        .bind(size)
        .bind(page * size)
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(Meeting::to_response).collect())
    }

    pub async fn create(pool: &PgPool, name: &str, snowflake: &str, state: &str) -> sqlx::Result<Meeting> {
        sqlx::query_as::<_, Meeting>(
            r#"INSERT INTO meeting (name, snowflake, state, active)
               VALUES ($1, $2, $3, true)
               RETURNING id, name, snowflake, state, active"#,
        )
        .bind(name)
        .bind(snowflake)
        .bind(state)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &PgPool, name: &str, state: &str, snowflake: &str) -> sqlx::Result<MeetingResponse> {
        let meeting = sqlx::query_as::<_, Meeting>(
            r#"UPDATE meeting
               SET name = $1, state = $2, active = true
               WHERE snowflake = $3
               RETURNING id, name, snowflake, state, active"#,
        )
        .bind(name)
        .bind(state)
        .bind(snowflake)
        .fetch_one(pool)
        .await?;
        Ok(meeting.to_response())
    }

    pub async fn soft_delete(pool: &PgPool, id: i64) -> sqlx::Result<u64> {
        let result = sqlx::query(
            r#"UPDATE meeting SET active = false WHERE id = $1 AND active = true"#,
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}
