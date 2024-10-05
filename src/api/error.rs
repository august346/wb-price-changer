use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    // #[error("Database error: {0}")]
    // DbError(#[from] sqlx::Error),

    // #[error("Invalid input: {0}")]
    // InvalidInput(String),

    #[error("No permission: {0}")]
    NoPermission(String),

    // #[error("Resource not found")]
    // NotFound,

    #[error("Internal server error")]
    InternalServerError,
}

impl AppError {
    pub fn unexpected(text: &str) -> AppError {
        error!("{}", text);
        AppError::InternalServerError
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            // AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NoPermission(msg) => (StatusCode::FORBIDDEN, msg),
            // AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}