use crate::error::AppError;
use crate::models::{OpenWeatherResponse, WeatherData};
use chrono::Utc;
use log::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct WeatherClient {
    api_key: String,
    client: reqwest::Client,
}

impl WeatherClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_weather(&self, city: &str, country: &str) -> Result<WeatherData, AppError> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={},{}&appid={}&units=metric",
            city, country, self.api_key
        );

        info!("🌐 Fetching weather from API: {}, {}", city, country);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::WeatherApi(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::WeatherApi(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        let weather_response: OpenWeatherResponse = response
            .json()
            .await
            .map_err(|e| AppError::WeatherApi(format!("Failed to parse response: {}", e)))?;

        let weather_data = WeatherData {
            id: Uuid::new_v4(),
            city: weather_response.name.clone(),
            country: weather_response.sys.country.clone(),
            temperature: weather_response.main.temp,
            feels_like: weather_response.main.feels_like,
            conditions: weather_response.weather.first()
                .map(|w| w.main.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            description: weather_response.weather.first()
                .map(|w| w.description.clone())
                .unwrap_or_else(|| "No description".to_string()),
            humidity: weather_response.main.humidity,
            wind_speed: weather_response.wind.speed,
            pressure: weather_response.main.pressure,
            fetched_at: Utc::now(),
        };

        info!(
            "✅ Weather fetched: {} - {}°C, {}",
            weather_data.city, weather_data.temperature, weather_data.conditions
        );

        Ok(weather_data)
    }

    pub async fn get_forecast(&self, city: &str, country: &str) -> Result<String, AppError> {
        // Optional: Implement 5-day forecast
        // Requires different API endpoint
        Ok(format!("Forecast for {}, {} (not implemented)", city, country))
    }
}