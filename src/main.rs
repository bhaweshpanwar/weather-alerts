use actix_files as fs;
use actix_web::{middleware, web, App, HttpServer};
use clap::{Parser, Subcommand};
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

mod config;
mod db;
mod email;
mod error;
mod handlers;
mod models;
mod weather;

use crate::config::Config;
use crate::db::Database;
use crate::error::AppError;

#[derive(Parser, Debug)]
#[command(author, version, about = "Weather Alert System - CRON Job Scheduler")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the web server with CRON scheduler
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Manually fetch weather for all users
    FetchWeather,
    /// Send test email
    TestEmail {
        #[arg(short, long)]
        to: String,
    },
    /// Initialize database schema
    InitDb,
    /// List all scheduled jobs
    ListJobs,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
    pub weather_client: weather::WeatherClient,
    pub email_client: email::EmailClient,
}

#[actix_web::main]
async fn main() -> Result<(), AppError> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenv::dotenv().ok();

    let cli = Cli::parse();
    let config = Config::from_env()?;

    info!("🚀 Weather Alert System Starting...");

    // Initialize database
    let db = Database::new(&config.database_url).await?;
    let weather_client = weather::WeatherClient::new(config.weather_api_key.clone());
    let email_client = email::EmailClient::new(
        &config.smtp_host,
        config.smtp_port,
        &config.smtp_username,
        &config.smtp_password,
    )?;

    match cli.command {
        Some(Commands::Serve { port }) => {
            start_server(port, db, config, weather_client, email_client).await?;
        }
        Some(Commands::FetchWeather) => {
            info!("📡 Manually fetching weather...");
            fetch_and_alert(&db, &weather_client, &email_client).await?;
            info!("✅ Weather fetch completed!");
        }
        Some(Commands::TestEmail { to }) => {
            info!("📧 Sending test email to {}", to);
            email_client
                .send_test_email(&to, "Weather Alert Test")
                .await?;
            info!("✅ Test email sent!");
        }
        Some(Commands::InitDb) => {
            info!("🗄️  Initializing database schema...");
            db.init_schema().await?;
            info!("✅ Database schema created!");
        }
        Some(Commands::ListJobs) => {
            list_jobs();
        }
        None => {
            start_server(8080, db, config, weather_client, email_client).await?;
        }
    }

    Ok(())
}

async fn start_server(
    port: u16,
    db: Database,
    config: Config,
    weather_client: weather::WeatherClient,
    email_client: email::EmailClient,
) -> Result<(), AppError> {
    info!("🌐 Starting server on http://0.0.0.0:{}", port);

    let scheduler = JobScheduler::new().await?;
    let scheduler = Arc::new(Mutex::new(scheduler));

    // Setup CRON job
    setup_weather_cron(
        scheduler.clone(),
        db.clone(),
        weather_client.clone(),
        email_client.clone(),
    )
    .await?;

    {
        let sched = scheduler.lock().await;
        sched.start().await?;
        info!("⏰ CRON scheduler started - Weather fetch every 2 hours");
    }

    let app_state = AppState {
        db: db.clone(),
        config: config.clone(),
        weather_client,
        email_client,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(handlers::configure_routes)
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

async fn setup_weather_cron(
    scheduler: Arc<Mutex<JobScheduler>>,
    db: Database,
    weather_client: weather::WeatherClient,
    email_client: email::EmailClient,
) -> Result<(), AppError> {
    let sched = scheduler.lock().await;

    // Run every 2 hours: "0 0 */2 * * *"
    // For testing every 5 minutes: "0 */5 * * * *"
    let job = Job::new_async("0 0 */2 * * *", move |_uuid, _l| {
        let db = db.clone();
        let weather_client = weather_client.clone();
        let email_client = email_client.clone();

        Box::pin(async move {
            info!("🌤️  CRON Job: Starting weather fetch...");
            match fetch_and_alert(&db, &weather_client, &email_client).await {
                Ok(_) => info!("✅ CRON Job: Weather fetch completed successfully"),
                Err(e) => log::error!("❌ CRON Job: Weather fetch failed: {}", e),
            }
        })
    })?;

    sched.add(job).await?;
    info!("✅ CRON job scheduled: Weather fetch every 2 hours");

    Ok(())
}

async fn fetch_and_alert(
    db: &Database,
    weather_client: &weather::WeatherClient,
    email_client: &email::EmailClient,
) -> Result<(), AppError> {
    // Get all unique cities from users
    let cities = db.get_all_user_cities().await?;
    info!("📍 Found {} unique cities to fetch", cities.len());

    for city_info in cities {
        info!("🌍 Fetching weather for {}, {}", city_info.city, city_info.country);

        // Fetch weather from API
        match weather_client.get_weather(&city_info.city, &city_info.country).await {
            Ok(weather) => {
                // Store in database
                db.store_weather_data(&weather).await?;
                info!(
                    "💾 Stored weather: {} - {}°C, {}",
                    city_info.city, weather.temperature, weather.conditions
                );

                // Check users in this city for alerts
                let users = db.get_users_by_city(&city_info.city).await?;
                
                for user in users {
                    if let Some(prefs) = db.get_user_preferences(user.id).await? {
                        let should_alert = check_alert_conditions(&weather, &prefs);
                        
                        if let Some(alert_message) = should_alert {
                            info!("🔔 Sending alert to {}: {}", user.email, alert_message);
                            
                            match email_client
                                .send_weather_alert(&user.email, &city_info.city, &alert_message)
                                .await
                            {
                                Ok(_) => {
                                    db.log_alert(user.id, "temperature", &alert_message).await?;
                                    info!("✅ Alert sent to {}", user.email);
                                }
                                Err(e) => {
                                    log::error!("❌ Failed to send alert to {}: {}", user.email, e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("❌ Failed to fetch weather for {}: {}", city_info.city, e);
            }
        }

        // Rate limiting - be nice to the API
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

fn check_alert_conditions(
    weather: &models::WeatherData,
    prefs: &models::UserPreferences,
) -> Option<String> {
    let temp = weather.temperature;
    let conditions = weather.conditions.to_lowercase();

    if let Some(max_temp) = prefs.max_temp {
        if temp > max_temp as f64 {
            return Some(format!(
                "🌡️ High temperature alert! Current: {:.1}°C (Your limit: {}°C)",
                temp, max_temp
            ));
        }
    }

    if let Some(min_temp) = prefs.min_temp {
        if temp < min_temp as f64 {
            return Some(format!(
                "🥶 Low temperature alert! Current: {:.1}°C (Your limit: {}°C)",
                temp, min_temp
            ));
        }
    }

    if prefs.alert_on_rain && conditions.contains("rain") {
        return Some(format!("☔ Rain alert! Current conditions: {}", weather.conditions));
    }

    if prefs.alert_on_snow && conditions.contains("snow") {
        return Some(format!("❄️ Snow alert! Current conditions: {}", weather.conditions));
    }

    if prefs.alert_on_storm && (conditions.contains("storm") || conditions.contains("thunder")) {
        return Some(format!("⚡ Storm alert! Current conditions: {}", weather.conditions));
    }

    None
}

fn list_jobs() {
    println!("📋 Scheduled CRON Jobs:");
    println!("  ⏰ Weather Fetch: Every 2 hours (0 0 */2 * * *)");
    println!("\n🔧 Manual Commands:");
    println!("  cargo run -- fetch-weather    (Manually fetch weather now)");
    println!("  cargo run -- init-db          (Initialize database)");
    println!("  cargo run -- test-email       (Send test email)");
}