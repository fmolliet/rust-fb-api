use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message): (StatusCode, String) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Database(_) => {
                tracing::error!("Database error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, String::from("Database error"))
            }
            AppError::Internal(_) => {
                tracing::error!("Internal error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal error"))
            }
        };

        (status, Json(json!({"error": message}))).into_response()
    }
}
