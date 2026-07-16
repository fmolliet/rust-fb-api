use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use uuid::Uuid;

use crate::db::AppState;
use crate::dto::register::{CreateRegisterDto, UpdateRegisterDto};
use crate::error::AppError;
use crate::models::register::Register;

pub fn register_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::post(create_register))
        .route("/open/{applicant_id}", axum::routing::get(get_open_registration))
        .route("/{id}", axum::routing::get(get_registration))
        .route("/{applicant_id}/finish", axum::routing::patch(finish_registration))
        .route("/{id}", axum::routing::delete(delete_registration))
}

async fn create_register(
    State(state): State<AppState>,
    Json(dto): Json<CreateRegisterDto>,
) -> Result<impl IntoResponse, AppError> {
    let existing = Register::find_unfinished_by_applicant_id(&state.pool, &dto.applicant_id)
        .await
        .map_err(|e| {
            tracing::error!("DB error checking unfinished registration: {}", e);
            AppError::Internal(e.into())
        })?;

    if let Some(reg) = existing {
        return Err(AppError::BadRequest(format!(
            "Open registration already exists for applicant: {}",
            reg.applicant_id
        )));
    }

    let reg = Register::create(&state.pool, &dto)
        .await
        .map_err(|e| {
            tracing::error!("DB error creating registration: {}", e);
            AppError::Database(e)
        })?;

    Ok((StatusCode::CREATED, Json(reg)))
}

async fn get_open_registration(
    State(state): State<AppState>,
    Path(applicant_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let reg = Register::find_unfinished_by_applicant_id(&state.pool, &applicant_id)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding open registration: {}", e);
            AppError::Internal(e.into())
        })?;

    match reg {
        Some(reg) => Ok((StatusCode::OK, Json(reg)).into_response()),
        None => Err(AppError::NotFound(format!(
            "No open registration for applicant: {}",
            applicant_id
        ))),
    }
}

async fn get_registration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let uuid = Uuid::parse_str(&id).map_err(|_| {
        AppError::BadRequest(format!("Invalid UUID format: {}", id))
    })?;

    let reg = Register::find_by_id(&state.pool, uuid)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding registration: {}", e);
            AppError::Internal(e.into())
        })?;

    match reg {
        Some(reg) => Ok((StatusCode::OK, Json(reg)).into_response()),
        None => Err(AppError::NotFound(format!("Registration not found: {}", id))),
    }
}

async fn finish_registration(
    State(state): State<AppState>,
    Path(applicant_id): Path<String>,
    Json(dto): Json<UpdateRegisterDto>,
) -> Result<impl IntoResponse, AppError> {
    let reg = Register::finish(&state.pool, &applicant_id, &dto)
        .await
        .map_err(|e| {
            match e {
                sqlx::Error::RowNotFound => {
                    AppError::NotFound(format!(
                        "No open registration for applicant: {}",
                        applicant_id
                    ))
                }
                _ => {
                    tracing::error!("DB error finishing registration: {}", e);
                    AppError::Database(e)
                }
            }
        })?;

    Ok((StatusCode::ACCEPTED, Json(reg)).into_response())
}

async fn delete_registration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let uuid = Uuid::parse_str(&id).map_err(|_| {
        AppError::BadRequest(format!("Invalid UUID format: {}", id))
    })?;

    let deleted = Register::delete_by_id(&state.pool, uuid)
        .await
        .map_err(|e| {
            tracing::error!("DB error deleting registration: {}", e);
            AppError::Internal(e.into())
        })?;

    if !deleted {
        return Err(AppError::NotFound(format!("Registration not found: {}", id)));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}
