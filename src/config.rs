// ==================== config.rs ====================
use crate::error::AppError;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub weather_api_key: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| AppError::Config("DATABASE_URL not set".to_string()))?,
            weather_api_key: env::var("WEATHER_API_KEY")
                .map_err(|_| AppError::Config("WEATHER_API_KEY not set".to_string()))?,
            smtp_host: env::var("SMTP_HOST")
                .unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            smtp_username: env::var("SMTP_USERNAME")
                .map_err(|_| AppError::Config("SMTP_USERNAME not set".to_string()))?,
            smtp_password: env::var("SMTP_PASSWORD")
                .map_err(|_| AppError::Config("SMTP_PASSWORD not set".to_string()))?,
        })
    }
}

