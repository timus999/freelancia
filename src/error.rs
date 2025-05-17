use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum AppError {
    Validation(ValidationErrors),
    Database(String),
    Unauthorized(String),
    Server(String),
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::Validation(errors) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": errors.to_string() })),
            ),
            AppError::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": msg })),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": msg })),
            ),
            AppError::Server(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": msg })),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "BadRequest", "message": msg })),
            ),
        };

        (status, body).into_response()
    }
}