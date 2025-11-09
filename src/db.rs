use crate::error::AppError;
use crate::models::*;
use log::info;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        info!("🔌 Connecting to PostgreSQL database...");

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        info!("✅ Database connection established");

        Ok(Self { pool })
    }

    pub async fn init_schema(&self) -> Result<(), AppError> {
        info!("📋 Creating database schema...");

        // Each SQL command is now a separate string in the vector
        let queries = vec![
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                email VARCHAR(255) UNIQUE NOT NULL,
                city VARCHAR(100) NOT NULL,
                country VARCHAR(2) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
            "#,
            "CREATE INDEX IF NOT EXISTS idx_users_city ON users(city);",
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);",
            r#"
            CREATE TABLE IF NOT EXISTS user_preferences (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                min_temp INTEGER,
                max_temp INTEGER,
                alert_on_rain BOOLEAN DEFAULT false,
                alert_on_snow BOOLEAN DEFAULT false,
                alert_on_storm BOOLEAN DEFAULT false,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                UNIQUE(user_id)
            );
            "#,
            "CREATE INDEX IF NOT EXISTS idx_preferences_user_id ON user_preferences(user_id);",
            r#"
            CREATE TABLE IF NOT EXISTS weather_data (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                city VARCHAR(100) NOT NULL,
                country VARCHAR(2) NOT NULL,
                temperature DOUBLE PRECISION NOT NULL,
                feels_like DOUBLE PRECISION NOT NULL,
                conditions VARCHAR(100) NOT NULL,
                description TEXT,
                humidity INTEGER NOT NULL,
                wind_speed DOUBLE PRECISION NOT NULL,
                pressure INTEGER NOT NULL,
                fetched_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
            "#,
            "CREATE INDEX IF NOT EXISTS idx_weather_city ON weather_data(city);",
            "CREATE INDEX IF NOT EXISTS idx_weather_fetched_at ON weather_data(fetched_at DESC);",
            r#"
            CREATE TABLE IF NOT EXISTS alert_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                alert_type VARCHAR(50) NOT NULL,
                message TEXT NOT NULL,
                sent_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
            "#,
            "CREATE INDEX IF NOT EXISTS idx_alerts_user_id ON alert_logs(user_id);",
            "CREATE INDEX IF NOT EXISTS idx_alerts_sent_at ON alert_logs(sent_at DESC);",
        ];

        // The loop now executes each command individually
        for (i, query) in queries.iter().enumerate() {
            info!("Executing migration query {} of {}", i + 1, queries.len());
            sqlx::query(query).execute(&self.pool).await?;
        }

        info!("✅ Database schema created successfully");
        Ok(())
    }

    // ... the rest of the file is correct and does not need to be changed ...
    // User operations
    pub async fn create_user(&self, req: &CreateUserRequest) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, city, country)
            VALUES ($1, $2, UPPER($3))
            RETURNING *
            "#,
        )
        .bind(&req.email)
        .bind(&req.city)
        .bind(&req.country)
        .fetch_one(&self.pool)
        .await?;

        // Create default preferences
        sqlx::query(
            r#"
            INSERT INTO user_preferences (user_id)
            VALUES ($1)
            "#,
        )
        .bind(user.id)
        .execute(&self.pool)
        .await?;

        info!("✅ User created: {} - {}, {}", user.email, user.city, user.country);
        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn get_users_by_city(&self, city: &str) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE LOWER(city) = LOWER($1)
            "#,
        )
        .bind(city)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn get_all_user_cities(&self) -> Result<Vec<CityInfo>, AppError> {
        let cities = sqlx::query_as::<_, CityInfo>(
            r#"
            SELECT DISTINCT city, country FROM users
            ORDER BY city
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(cities)
    }

    // Preferences operations
    pub async fn get_user_preferences(&self, user_id: Uuid) -> Result<Option<UserPreferences>, AppError> {
        let prefs = sqlx::query_as::<_, UserPreferences>(
            r#"
            SELECT * FROM user_preferences WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(prefs)
    }

    pub async fn update_user_preferences(
        &self,
        user_id: Uuid,
        req: &UpdatePreferencesRequest,
    ) -> Result<UserPreferences, AppError> {
        let prefs = sqlx::query_as::<_, UserPreferences>(
            r#"
            UPDATE user_preferences
            SET
                min_temp = COALESCE($2, min_temp),
                max_temp = COALESCE($3, max_temp),
                alert_on_rain = COALESCE($4, alert_on_rain),
                alert_on_snow = COALESCE($5, alert_on_snow),
                alert_on_storm = COALESCE($6, alert_on_storm),
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(req.min_temp)
        .bind(req.max_temp)
        .bind(req.alert_on_rain)
        .bind(req.alert_on_snow)
        .bind(req.alert_on_storm)
        .fetch_one(&self.pool)
        .await?;

        info!("✅ Preferences updated for user: {}", user_id);
        Ok(prefs)
    }

    // Weather data operations
    pub async fn store_weather_data(&self, weather: &WeatherData) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO weather_data
            (city, country, temperature, feels_like, conditions, description,
             humidity, wind_speed, pressure, fetched_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&weather.city)
        .bind(&weather.country)
        .bind(weather.temperature)
        .bind(weather.feels_like)
        .bind(&weather.conditions)
        .bind(&weather.description)
        .bind(weather.humidity)
        .bind(weather.wind_speed)
        .bind(weather.pressure)
        .bind(weather.fetched_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_latest_weather(&self, city: &str) -> Result<Option<WeatherData>, AppError> {
        let weather = sqlx::query_as::<_, WeatherData>(
            r#"
            SELECT * FROM weather_data
            WHERE LOWER(city) = LOWER($1)
            ORDER BY fetched_at DESC
            LIMIT 1
            "#,
        )
        .bind(city)
        .fetch_optional(&self.pool)
        .await?;

        Ok(weather)
    }

    pub async fn get_weather_history(
        &self,
        city: &str,
        limit: i64,
    ) -> Result<Vec<WeatherData>, AppError> {
        let history = sqlx::query_as::<_, WeatherData>(
            r#"
            SELECT * FROM weather_data
            WHERE LOWER(city) = LOWER($1)
            ORDER BY fetched_at DESC
            LIMIT $2
            "#,
        )
        .bind(city)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    // Alert logs
    pub async fn log_alert(&self, user_id: Uuid, alert_type: &str, message: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO alert_logs (user_id, alert_type, message)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user_id)
        .bind(alert_type)
        .bind(message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_alerts(&self, user_id: Uuid, limit: i64) -> Result<Vec<AlertLog>, AppError> {
        let alerts = sqlx::query_as::<_, AlertLog>(
            r#"
            SELECT * FROM alert_logs
            WHERE user_id = $1
            ORDER BY sent_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(alerts)
    }

    pub async fn get_all_alerts(&self, limit: i64) -> Result<Vec<AlertLog>, AppError> {
        let alerts = sqlx::query_as::<_, AlertLog>(
            r#"
            SELECT * FROM alert_logs
            ORDER BY sent_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(alerts)
    }
}