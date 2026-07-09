use chrono::{Datelike, TimeZone};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

const TZ: chrono::FixedOffset = chrono::FixedOffset::west_opt(3 * 3600).unwrap();

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Birthday {
    pub snowflake: String,
    pub name: String,
    pub day: u8,
    pub month: u8,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BirthdayResponse {
    pub snowflake: String,
    pub name: String,
    pub day: u8,
    pub month: u8,
}

impl Birthday {
    pub fn today_day_month() -> (i32, i32) {
        let now = TZ.from_utc_datetime(&chrono::Utc::now().naive_utc());
        (now.day() as i32, now.month() as i32)
    }
}

impl BirthdayResponse {
    pub fn today_day_month() -> (i32, i32) {
        let now = TZ.from_utc_datetime(&chrono::Utc::now().naive_utc());
        (now.day() as i32, now.month() as i32)
    }
}
