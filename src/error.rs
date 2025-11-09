// ==================== error.rs ====================
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Scheduler(tokio_cron_scheduler::JobSchedulerError),
    WeatherApi(String),
    Email(String),
    Config(String),
    NotFound(String),
    Conflict(String),
    Validation(String),
    Internal(String),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::Scheduler(e) => write!(f, "Scheduler error: {}", e),
            AppError::WeatherApi(e) => write!(f, "Weather API error: {}", e),
            AppError::Email(e) => write!(f, "Email error: {}", e),
            AppError::Config(e) => write!(f, "Configuration error: {}", e),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        HttpResponse::build(status).json(serde_json::json!({
            "success": false,
            "error": message
        }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<tokio_cron_scheduler::JobSchedulerError> for AppError {
    fn from(err: tokio_cron_scheduler::JobSchedulerError) -> Self {
        AppError::Scheduler(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}