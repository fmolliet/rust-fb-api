use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::PgPool;
use crate::dto::register::{CreateRegisterDto, UpdateRegisterDto};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Register {
    pub id: String,
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

async fn query_create(pool: &PgPool, dto: &CreateRegisterDto) -> sqlx::Result<Register> {
    sqlx::query_as::<_, Register>(
        r#"INSERT INTO registrations (applicantid, birthday, specie, source, messageid, finished)
           VALUES ($1, $2, $3, $4, $5, FALSE)
           RETURNING *"#,
    )
    .bind(&dto.applicant_id)
    .bind(&dto.birthday)
    .bind(&dto.specie)
    .bind(&dto.source)
    .bind(&dto.message_id)
    .fetch_one(pool)
    .await
}

async fn query_find_unfinished(pool: &PgPool, applicant_id: &str) -> sqlx::Result<Option<Register>> {
    sqlx::query_as::<_, Register>(
        r#"SELECT * FROM registrations WHERE applicantid = $1 AND finished = FALSE LIMIT 1"#,
    )
    .bind(applicant_id)
    .fetch_optional(pool)
    .await
}

async fn query_find_by_id(pool: &PgPool, id: &str) -> sqlx::Result<Option<Register>> {
    sqlx::query_as::<_, Register>(
        r#"SELECT * FROM registrations WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

async fn query_finish(pool: &PgPool, applicant_id: &str, dto: &UpdateRegisterDto) -> sqlx::Result<Register> {
    sqlx::query_as::<_, Register>(
        r#"UPDATE registrations
           SET finished = TRUE, curatorid = $2, approved = $3, updatedat = current_timestamp
           WHERE applicantid = $1 AND finished = FALSE
           RETURNING *"#,
    )
    .bind(applicant_id)
    .bind(&dto.curator_id)
    .bind(dto.approved)
    .fetch_one(pool)
    .await
}

async fn query_delete_by_id(pool: &PgPool, id: &str) -> sqlx::Result<bool> {
    let result = sqlx::query(
        r#"DELETE FROM registrations WHERE id = $1"#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

impl Register {
    pub async fn create(pool: &PgPool, dto: &CreateRegisterDto) -> sqlx::Result<Register> {
        query_create(pool, dto).await
    }

    pub async fn find_unfinished_by_applicant_id(pool: &PgPool, applicant_id: &str) -> sqlx::Result<Option<Register>> {
        query_find_unfinished(pool, applicant_id).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &str) -> sqlx::Result<Option<Register>> {
        query_find_by_id(pool, id).await
    }

    pub async fn finish(pool: &PgPool, applicant_id: &str, dto: &UpdateRegisterDto) -> sqlx::Result<Register> {
        query_finish(pool, applicant_id, dto).await
    }

    pub async fn delete_by_id(pool: &PgPool, id: &str) -> sqlx::Result<bool> {
        query_delete_by_id(pool, id).await
    }
}
