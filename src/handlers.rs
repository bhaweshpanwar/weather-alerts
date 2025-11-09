use crate::error::AppError;
use crate::models::*;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use log::info;
use uuid::Uuid;
use validator::Validate;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/users")
                    .route("", web::post().to(create_user))
                    .route("", web::get().to(get_all_users))
                    .route("/{user_id}", web::get().to(get_user))
                    .route("/{user_id}/preferences", web::get().to(get_preferences))
                    .route("/{user_id}/preferences", web::put().to(update_preferences))
                    .route("/{user_id}/alerts", web::get().to(get_user_alerts)),
            )
            .service(
                web::scope("/weather")
                    .route("/current/{city}", web::get().to(get_current_weather))
                    .route("/history/{city}", web::get().to(get_weather_history))
                    .route("/fetch", web::post().to(manual_fetch_weather)),
            )
            .service(
                web::scope("/alerts")
                    .route("", web::get().to(get_all_alerts)),
            ),
    );
}

// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "Weather Alert System",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// User endpoints
async fn create_user(
    state: web::Data<AppState>,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if user already exists
    if let Some(_existing) = state.db.get_user_by_email(&req.email).await? {
        return Err(AppError::Conflict("User with this email already exists".to_string()));
    }

    let user = state.db.create_user(&req).await?;

    // Send welcome email
    tokio::spawn({
        let email_client = state.email_client.clone();
        let user_email = user.email.clone();
        let city = user.city.clone();
        async move {
            if let Err(e) = email_client.send_welcome_email(&user_email, &city).await {
                log::error!("Failed to send welcome email: {}", e);
            }
        }
    });

    Ok(HttpResponse::Created().json(ApiResponse::success(
        user,
        "User registered successfully. Welcome email sent!",
    )))
}

async fn get_all_users(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let users = state.db.get_all_users().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(users, "Users fetched successfully")))
}

async fn get_user(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user = state
        .db
        .get_user_by_id(*user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let preferences = state.db.get_user_preferences(*user_id).await?;

    let response = UserWithPreferences { user, preferences };

    Ok(HttpResponse::Ok().json(ApiResponse::success(response, "User fetched successfully")))
}

// Preferences endpoints
async fn get_preferences(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let preferences = state
        .db
        .get_user_preferences(*user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Preferences not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(preferences, "Preferences fetched")))
}

async fn update_preferences(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
    req: web::Json<UpdatePreferencesRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let preferences = state.db.update_user_preferences(*user_id, &req).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        preferences,
        "Preferences updated successfully",
    )))
}

// Weather endpoints
async fn get_current_weather(
    state: web::Data<AppState>,
    city: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let weather = state
        .db
        .get_latest_weather(&city)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No weather data found for {}", city)))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(weather, "Weather data fetched")))
}

async fn get_weather_history(
    state: web::Data<AppState>,
    city: web::Path<String>,
    query: web::Query<HistoryQuery>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(24); // Default 24 hours
    let history = state.db.get_weather_history(&city, limit).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(history, "Weather history fetched")))
}

async fn manual_fetch_weather(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    info!("🔄 Manual weather fetch triggered via API");

    // Spawn background task
    tokio::spawn({
        let db = state.db.clone();
        let weather_client = state.weather_client.clone();
        let email_client = state.email_client.clone();

        async move {
            match crate::fetch_and_alert(&db, &weather_client, &email_client).await {
                Ok(_) => info!("✅ Manual weather fetch completed"),
                Err(e) => log::error!("❌ Manual weather fetch failed: {}", e),
            }
        }
    });

    Ok(HttpResponse::Accepted().json(ApiResponse::<()>::error(
        "Weather fetch started in background",
    )))
}

// Alert endpoints
async fn get_user_alerts(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
    query: web::Query<AlertQuery>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(50);
    let alerts = state.db.get_user_alerts(*user_id, limit).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(alerts, "Alerts fetched")))
}

async fn get_all_alerts(
    state: web::Data<AppState>,
    query: web::Query<AlertQuery>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(100);
    let alerts = state.db.get_all_alerts(limit).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(alerts, "All alerts fetched")))
}

// Query parameters
#[derive(serde::Deserialize)]
struct HistoryQuery {
    limit: Option<i64>,
}

#[derive(serde::Deserialize)]
struct AlertQuery {
    limit: Option<i64>,
}