use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub city: String,
    pub country: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 2, message = "City name too short"))]
    pub city: String,
    #[validate(length(min = 2, max = 2, message = "Country code must be 2 letters (e.g., US, IN)"))]
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPreferences {
    pub id: Uuid,
    pub user_id: Uuid,
    pub min_temp: Option<i32>,
    pub max_temp: Option<i32>,
    pub alert_on_rain: bool,
    pub alert_on_snow: bool,
    pub alert_on_storm: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdatePreferencesRequest {
    pub min_temp: Option<i32>,
    pub max_temp: Option<i32>,
    pub alert_on_rain: Option<bool>,
    pub alert_on_snow: Option<bool>,
    pub alert_on_storm: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WeatherData {
    pub id: Uuid,
    pub city: String,
    pub country: String,
    pub temperature: f64,
    pub feels_like: f64,
    pub conditions: String,
    pub description: String,
    pub humidity: i32,
    pub wind_speed: f64,
    pub pressure: i32,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub alert_type: String,
    pub message: String,
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub city: String,
    pub country: String,
    pub temperature: f64,
    pub feels_like: f64,
    pub conditions: String,
    pub description: String,
    pub humidity: i32,
    pub wind_speed: f64,
    pub pressure: i32,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithPreferences {
    pub user: User,
    pub preferences: Option<UserPreferences>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.to_string(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: message.to_string(),
        }
    }
}

// OpenWeatherMap API Response
#[derive(Debug, Deserialize)]
pub struct OpenWeatherResponse {
    pub main: MainWeather,
    pub weather: Vec<Weather>,
    pub wind: Wind,
    pub name: String,
    pub sys: Sys,
}

#[derive(Debug, Deserialize)]
pub struct MainWeather {
    pub temp: f64,
    pub feels_like: f64,
    pub humidity: i32,
    pub pressure: i32,
}

#[derive(Debug, Deserialize)]
pub struct Weather {
    pub main: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Wind {
    pub speed: f64,
}

#[derive(Debug, Deserialize)]
pub struct Sys {
    pub country: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CityInfo {
    pub city: String,
    pub country: String,
}