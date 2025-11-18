use axum::{response::{IntoResponse, Json}, http::StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Invalid payload: {0}")]
    Validation(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
