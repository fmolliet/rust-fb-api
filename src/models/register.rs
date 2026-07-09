use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
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
