use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, debug_handler};
use serde::Deserialize;
use sqlx::types::Uuid;
use validator::Validate;

use crate::db::AppState;
use crate::dto::announce::{CreateAnnouncementDto, UpdateAnnouncementDto};
use crate::error::AppError;
use crate::models::announce::Announcement;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub size: Option<i64>,
}

pub fn announce_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list_announcements).post(create_announcement))
        .route("/{id}", axum::routing::get(get_announcement))
        .route("/{id}", axum::routing::patch(update_announcement))
        .route("/{id}", axum::routing::delete(delete_announcement))
}

#[debug_handler]
async fn list_announcements(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(0);
    let size = params.size.unwrap_or(100);

    let announcements = Announcement::list_paginated(&state.pool, page, size).await?;
    Ok((StatusCode::OK, Json(announcements)).into_response())
}

#[debug_handler]
async fn get_announcement(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let announcement = Announcement::find_by_id_response(&state.pool, &id.to_string()).await?;

    match announcement {
        Some(a) => Ok((StatusCode::OK, Json(a)).into_response()),
        None => Err(AppError::NotFound(format!("Announcement not found: {}", id))),
    }
}

#[debug_handler]
async fn create_announcement(
    State(state): State<AppState>,
    Json(dto): Json<CreateAnnouncementDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate().map_err(|e| {
        let details: Vec<_> = e.field_errors()
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.iter().map(|s| s.code.as_ref()).collect::<Vec<_>>().join(", ")))
            .collect();
        AppError::BadRequest(format!("Validation failed: {}", details.join("; ")))
    })?;

    let channel_id = dto.channel_id.unwrap_or_default();
    let message = dto.message.unwrap_or_default();
    let frequency = dto.frequency.as_deref().unwrap_or("daily");

    let announcement = Announcement::create(&state.pool, &channel_id, &message, frequency).await
        .map_err(|e| {
            tracing::error!("DB error creating announcement: {}", e);
            AppError::Internal(e.into())
        })?;

    let response = announcement.to_response();
    Ok((StatusCode::CREATED, Json(response)).into_response())
}

#[debug_handler]
async fn update_announcement(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateAnnouncementDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate().map_err(|e| {
        let details: Vec<_> = e.field_errors()
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.iter().map(|s| s.code.as_ref()).collect::<Vec<_>>().join(", ")))
            .collect();
        AppError::BadRequest(format!("Validation failed: {}", details.join("; ")))
    })?;

    let channel_id = dto.channel_id.as_deref();
    let message = dto.message.as_deref();
    let frequency = dto.frequency.as_deref();

    let result = Announcement::update(&state.pool, &id.to_string(), channel_id, message, frequency).await?;

    match result {
        Some(a) => Ok((StatusCode::OK, Json(a)).into_response()),
        None => Err(AppError::NotFound(format!("Announcement not found: {}", id))),
    }
}

#[debug_handler]
async fn delete_announcement(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let affected = Announcement::delete(&state.pool, &id.to_string()).await?;

    if affected == 0 {
        return Err(AppError::NotFound(format!("Announcement not found: {}", id)));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}
