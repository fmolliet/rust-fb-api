use fbrustapi::error::AppError;
use axum::response::IntoResponse;
use axum::http::StatusCode;

#[test]
fn test_not_found_error_status() {
    let err = AppError::NotFound("User not found".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_conflict_error_status() {
    let err = AppError::Conflict("Email already exists".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[test]
fn test_bad_request_error_status() {
    let err = AppError::BadRequest("Invalid parameters".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_database_error_status() {
    let err = AppError::Database(sqlx::Error::RowNotFound);
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_internal_error_status() {
    let err = AppError::Internal(anyhow::anyhow!("Something went wrong"));
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
