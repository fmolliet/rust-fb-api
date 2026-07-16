use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::db::AppState;
use crate::dto::birthday::BirthdayRequestDto;
use crate::error::AppError;
use crate::models::birthday::{Birthday, BirthdayResponse};

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub size: Option<i64>,
}

pub fn birthday_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list_birthdays).post(create_birthday))
        .route("/today", axum::routing::get(get_today_birthdays))
        .route("/month", axum::routing::get(get_month_birthdays))
        .route("/{snowflake}", axum::routing::get(get_birthday)
            .patch(update_birthday)
            .delete(delete_birthday))
}

async fn list_birthdays(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(0);
    let size = params.size.unwrap_or(10);

    let birthdays = Birthday::list_paginated(&state.pool, page, size)
        .await
        .map_err(|e| {
            tracing::error!("DB error listing birthdays: {}", e);
            AppError::Internal(e.into())
        })?;

    let responses: Vec<BirthdayResponse> = birthdays
        .into_iter()
        .map(|b| BirthdayResponse {
            snowflake: b.snowflake,
            name: b.name,
            day: b.day,
            month: b.month,
        })
        .collect();

    Ok((StatusCode::OK, Json(responses)).into_response())
}

async fn get_today_birthdays(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let birthdays = Birthday::find_today(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding today birthdays: {}", e);
            AppError::Internal(e.into())
        })?;

    Ok((StatusCode::OK, Json(birthdays)).into_response())
}

async fn get_month_birthdays(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let birthdays = Birthday::find_month(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding month birthdays: {}", e);
            AppError::Internal(e.into())
        })?;

    Ok((StatusCode::OK, Json(birthdays)).into_response())
}

async fn get_birthday(
    State(state): State<AppState>,
    Path(snowflake): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let birthday = Birthday::find_by_snowflake(&state.pool, &snowflake)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding birthday: {}", e);
            AppError::Internal(e.into())
        })?;

    match birthday {
        Some(b) => {
            let response = BirthdayResponse {
                snowflake: b.snowflake,
                name: b.name,
                day: b.day,
                month: b.month,
            };
            Ok((StatusCode::OK, Json(response)).into_response())
        }
        None => Err(AppError::NotFound(format!("Birthday not found: {}", snowflake))),
    }
}

async fn create_birthday(
    State(state): State<AppState>,
    Json(dto): Json<BirthdayRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let birthday = Birthday::create(&state.pool, &dto)
        .await
        .map_err(|e| {
            match e {
                sqlx::Error::Database(ref d) if d.code().as_deref() == Some("23505") => {
                    AppError::BadRequest(format!("Birthday already exists for snowflake: {}", dto.snowflake))
                }
                _ => {
                    tracing::error!("DB error creating birthday: {}", e);
                    AppError::Database(e)
                }
            }
        })?;

    Ok((StatusCode::CREATED, Json(birthday)).into_response())
}

async fn update_birthday(
    State(state): State<AppState>,
    Path(snowflake): Path<String>,
    Json(dto): Json<BirthdayRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let birthday = Birthday::update(&state.pool, &snowflake, &dto)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                AppError::NotFound(format!("Birthday not found: {}", snowflake))
            }
            _ => {
                tracing::error!("DB error updating birthday: {}", e);
                AppError::Database(e)
            }
        })?;

    Ok((StatusCode::ACCEPTED, Json(birthday)).into_response())
}

async fn delete_birthday(
    State(state): State<AppState>,
    Path(snowflake): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = Birthday::delete_by_snowflake(&state.pool, &snowflake)
        .await
        .map_err(|e| {
            tracing::error!("DB error deleting birthday: {}", e);
            AppError::Internal(e.into())
        })?;

    if !deleted {
        return Err(AppError::NotFound(format!("Birthday not found: {}", snowflake)));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}
