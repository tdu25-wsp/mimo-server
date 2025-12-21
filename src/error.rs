use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    DatabaseError(String),
    ValidationError(String),
    HashingError(String),
    EnvironmentError(String),
    ExternalServiceError(String), // 外部サービス関連のエラー
    ConfigError(String), // APIキー設定エラー
    AuthenticationError(String),
    Unauthorized(String),
    Forbidden(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::HashingError(msg) => write!(f, "Hashing error: {}", msg),
            AppError::EnvironmentError(msg) => write!(f, "Environment error: {}", msg),
            AppError::ExternalServiceError(msg) => write!(f, "External service error: {}", msg), // 外部サービス関連のエラー
            AppError::ConfigError(msg) => write!(f, "Configuration error: {}", msg), // APIキー設定エラー
            AppError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::HashingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::EnvironmentError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ConfigError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ExternalServiceError(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
