use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, debug_handler};
use serde::Deserialize;
use validator::Validate;

use crate::db::AppState;
use crate::dto::meeting::{CreateMeetingDto, UpdateMeetingDto};
use crate::error::AppError;
use crate::models::meeting::Meeting;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub size: Option<i64>,
}

pub fn meeting_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list_meetings).post(create_meeting).put(update_meeting))
        .route("/state/{state}", axum::routing::get(list_meetings_by_state))
        .route("/{id}", axum::routing::get(get_meeting).delete(delete_meeting))
}

#[debug_handler]
async fn list_meetings(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(0);
    let size = params.size.unwrap_or(25);

    let meetings = Meeting::list_active(&state.pool, page, size).await?;
    Ok((StatusCode::OK, Json(meetings)).into_response())
}

#[debug_handler]
async fn get_meeting(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let meeting = Meeting::find_by_snowflake_response(&state.pool, &id).await?;

    match meeting {
        Some(m) => Ok((StatusCode::OK, Json(m)).into_response()),
        None => Err(AppError::NotFound(format!("Meeting not found: {}", id))),
    }
}

#[debug_handler]
async fn list_meetings_by_state(
    State(state): State<AppState>,
    Path(state_code): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(0);
    let size = params.size.unwrap_or(25);

    let meetings = Meeting::list_by_state(&state.pool, &state_code, page, size).await?;
    Ok((StatusCode::OK, Json(meetings)).into_response())
}

#[debug_handler]
async fn create_meeting(
    State(state): State<AppState>,
    Json(dto): Json<CreateMeetingDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate().map_err(|e| {
        let details: Vec<_> = e.field_errors()
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.iter().map(|s| s.code.as_ref()).collect::<Vec<_>>().join(", ")))
            .collect();
        AppError::BadRequest(format!("Validation failed: {}", details.join("; ")))
    })?;

    let name = dto.name.as_deref().unwrap_or("");
    let meeting = Meeting::create(&state.pool, name, &dto.snowflake, &dto.state).await
        .map_err(|e| {
            match &e {
                sqlx::Error::Database(ref d) if d.code().as_deref() == Some("23505") => {
                    AppError::Conflict(format!("Meeting with snowflake '{}' already exists", dto.snowflake))
                }
                _ => {
                    tracing::error!("DB error creating meeting: {}", e);
                    AppError::Internal(e.into())
                }
            }
        })?;

    let response = meeting.to_response();
    Ok((StatusCode::CREATED, Json(response)).into_response())
}

#[debug_handler]
async fn update_meeting(
    State(state): State<AppState>,
    Json(dto): Json<UpdateMeetingDto>,
) -> Result<impl IntoResponse, AppError> {
    dto.validate().map_err(|e| {
        let details: Vec<_> = e.field_errors()
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.iter().map(|s| s.code.as_ref()).collect::<Vec<_>>().join(", ")))
            .collect();
        AppError::BadRequest(format!("Validation failed: {}", details.join("; ")))
    })?;

    let existing = match Meeting::find_by_snowflake(&state.pool, &dto.snowflake).await? {
        Some(m) => m,
        None => return Err(AppError::NotFound(format!("Meeting not found: {}", dto.snowflake))),
    };

    let name_str = match &dto.name {
        Some(ref n) if !n.is_empty() => n.as_str(),
        _ => &existing.name,
    };

    let updated = Meeting::update(&state.pool, name_str, &dto.state, &dto.snowflake).await
        .map_err(|e| {
            match &e {
                sqlx::Error::RowNotFound => {
                    AppError::NotFound(format!("Meeting not found: {}", dto.snowflake))
                }
                _ => {
                    tracing::error!("DB error updating meeting: {}", e);
                    AppError::Internal(e.into())
                }
            }
        })?;

    Ok((StatusCode::OK, Json(updated)).into_response())
}

#[debug_handler]
async fn delete_meeting(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let affected = Meeting::soft_delete(&state.pool, id).await?;

    if affected == 0 {
        return Err(AppError::NotFound(format!("Meeting not found: {}", id)));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}
