# 🌤️ Weather Alert System

> A scalable, production-ready weather notification service built with Rust, Actix-web, and PostgreSQL. Automatically fetches weather data every 2 hours via CRON jobs and sends personalized email alerts based on user preferences.

[![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)](https://www.rust-lang.org/)
[![Actix](https://img.shields.io/badge/actix--web-4.9-blue.svg)](https://actix.rs/)
[![PostgreSQL](https://img.shields.io/badge/postgresql-16-blue.svg)](https://www.postgresql.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## Features

- **Automated CRON Jobs** - Fetches weather data every 2 hours
- **Smart Email Alerts** - HTML-formatted notifications when conditions match user preferences
- **Multi-City Support** - Track weather for unlimited locations
- **RESTful API** - Complete CRUD operations for users and preferences
- **Secure Configuration** - Environment-based secrets management
- **Weather History** - Store and query historical weather data
- **Personalized Alerts** - Temperature thresholds, rain, snow, and storm notifications
- **Production Ready** - Systemd service, logging, error handling
- **POSIX CLI** - Manual job execution via command-line interface
- **AWS Compatible** - Easy deployment to EC2 + RDS

## Architecture

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│   User API  │─────▶│  Actix-web   │─────▶│ PostgreSQL  │
│  (REST)     │      │   Server     │      │  Database   │
└─────────────┘      └──────────────┘      └─────────────┘
                            │
                            │ CRON Job
                            │ (Every 2h)
                            ▼
                     ┌──────────────┐
                     │ OpenWeather  │
                     │     API      │
                     └──────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │    SMTP      │
                     │   (Email)    │
                     └──────────────┘
```

## Prerequisites

- **Rust** 1.82+ ([Install](https://rustup.rs/))
- **PostgreSQL** 14+ ([Install](https://www.postgresql.org/download/))
- **OpenWeatherMap API Key** - Free tier available ([Get Key](https://openweathermap.org/api))
- **Gmail Account** with App Password ([Setup 2FA](https://myaccount.google.com/apppasswords))

## Quick Start

### 1. Clone & Setup

```bash
# Clone the repository
git clone https://github.com/bhaweshpanwar/weather-alert-system.git
cd weather-alert-system

# Install dependencies (handled by Cargo)
cargo build
```

### 2. Database Setup

```bash
# Create database
createdb weather_alerts

# Or using psql
psql -U postgres
CREATE DATABASE weather_alerts;
CREATE USER weather_user WITH ENCRYPTED PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE weather_alerts TO weather_user;
\q
```

### 3. Configuration

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` with your credentials:

```env
DATABASE_URL=postgres://weather_user:password@localhost:5432/weather_alerts
WEATHER_API_KEY=your_openweathermap_api_key
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_16_char_app_password
RUST_LOG=weather_alert_system=info,actix_web=info
```

### 4. Initialize Database

```bash
cargo run -- init-db
```

**Output:**

```
🔌 Connecting to PostgreSQL database...
✅ Database connection established
📋 Creating database schema...
✅ Database schema created successfully
```

### 5. Test Email Configuration

```bash
cargo run -- test-email --to your_email@gmail.com
```

You should receive a test email within 30 seconds.

### 6. Start the Server

```bash
cargo run -- serve --port 8080
```

**Server starts at:** `http://localhost:8080`

## API Documentation

### Base URL

```
http://localhost:8080/api
```

### Endpoints

#### Health Check

```http
GET /api/health
```

**Response:**

```json
{
  "status": "healthy",
  "service": "Weather Alert System",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

#### Register User

```http
POST /api/users
Content-Type: application/json

{
  "email": "user@example.com",
  "city": "London",
  "country": "GB"
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "city": "London",
    "country": "GB",
    "created_at": "2024-01-01T12:00:00Z"
  },
  "message": "User registered successfully. Welcome email sent!"
}
```

#### Update Preferences

```http
PUT /api/users/{user_id}/preferences
Content-Type: application/json

{
  "min_temp": 10,
  "max_temp": 30,
  "alert_on_rain": true,
  "alert_on_snow": true,
  "alert_on_storm": true
}
```

#### Get Current Weather

```http
GET /api/weather/current/{city}
```

**Example:**

```bash
curl http://localhost:8080/api/weather/current/London
```

#### Get Weather History

```http
GET /api/weather/history/{city}?limit=24
```

#### Manually Trigger Weather Fetch

```http
POST /api/weather/fetch
```

#### Get User Alerts

```http
GET /api/users/{user_id}/alerts?limit=50
```

#### Get All Users

```http
GET /api/users
```

#### Get User Details

```http
GET /api/users/{user_id}
```

#### Get All Alerts

```http
GET /api/alerts?limit=100
```

For complete API examples, see [API_EXAMPLES.md](docs/API_EXAMPLES.md)

## CLI Commands

The application provides a POSIX-compliant CLI for manual operations:

```bash
# Start server with custom port
cargo run -- serve --port 3000

# Manually fetch weather for all users (runs CRON job immediately)
cargo run -- fetch-weather

# Send test email
cargo run -- test-email --to recipient@example.com

# Initialize database schema
cargo run -- init-db

# List all scheduled CRON jobs
cargo run -- list-jobs

# Show help
cargo run -- --help
```

## CRON Schedule

The system automatically fetches weather data using this schedule:

```
0 0 */2 * * *
```

**Translation:** Every 2 hours at minute 0, second 0

**Execution times:**

- 00:00, 02:00, 04:00, 06:00, 08:00, 10:00
- 12:00, 14:00, 16:00, 18:00, 20:00, 22:00

### How It Works

1. CRON job triggers every 2 hours
2. Fetches all unique cities from registered users
3. Calls OpenWeatherMap API for each city
4. Stores weather data in PostgreSQL
5. Checks each user's preferences
6. Sends email alerts if conditions match user thresholds

### Modifying the Schedule

Edit `src/main.rs`, find this line:

```rust
let job = Job::new_async("0 0 */2 * * *", move |_uuid, _l| {
```

**Common schedules:**

| Schedule         | Description               |
| ---------------- | ------------------------- |
| `0 0 */2 * * *`  | Every 2 hours (default)   |
| `0 0 */1 * * *`  | Every hour                |
| `0 */30 * * * *` | Every 30 minutes          |
| `0 */5 * * * *`  | Every 5 minutes (testing) |
| `0 0 6,18 * * *` | Twice daily (6 AM, 6 PM)  |

[Learn CRON syntax](https://crontab.guru/)

## 📂 Project Structure

```
weather-alert-system/
├── src/
│   ├── main.rs           # Entry point, server setup, CRON scheduling
│   ├── models.rs         # Data structures and types
│   ├── db.rs             # Database operations (CRUD)
│   ├── weather.rs        # Weather API client
│   ├── email.rs          # Email client (SMTP)
│   ├── handlers.rs       # API route handlers
│   ├── config.rs         # Configuration management
│   └── error.rs          # Error types and handling
├── Cargo.toml            # Rust dependencies
├── .env                  # Environment variables (create from .env.example)
├── .env.example          # Environment template
├── README.md             # This file
└── docs/
    ├── SETUP_GUIDE.md    # Detailed setup instructions
    ├── TESTING_GUIDE.md  # Testing scenarios
    ├── AWS_DEPLOY.md     # AWS deployment guide
    └── API_EXAMPLES.md   # API usage examples
```

## Database Schema

### Users Table

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    city VARCHAR(100) NOT NULL,
    country VARCHAR(2) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE
);
```

### User Preferences Table

```sql
CREATE TABLE user_preferences (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    min_temp INTEGER,
    max_temp INTEGER,
    alert_on_rain BOOLEAN,
    alert_on_snow BOOLEAN,
    alert_on_storm BOOLEAN,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
);
```

### Weather Data Table

```sql
CREATE TABLE weather_data (
    id UUID PRIMARY KEY,
    city VARCHAR(100) NOT NULL,
    country VARCHAR(2) NOT NULL,
    temperature DOUBLE PRECISION NOT NULL,
    feels_like DOUBLE PRECISION NOT NULL,
    conditions VARCHAR(100) NOT NULL,
    description TEXT,
    humidity INTEGER,
    wind_speed DOUBLE PRECISION,
    pressure INTEGER,
    fetched_at TIMESTAMP WITH TIME ZONE
);
```

### Alert Logs Table

```sql
CREATE TABLE alert_logs (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    alert_type VARCHAR(50),
    message TEXT,
    sent_at TIMESTAMP WITH TIME ZONE
);
```

## Testing

### Run Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# Run specific test
cargo test test_name
```

### Manual Testing

```bash
# 1. Register a test user
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","city":"London","country":"GB"}'

# 2. Set preferences
curl -X PUT http://localhost:8080/api/users/{user_id}/preferences \
  -H "Content-Type: application/json" \
  -d '{"max_temp":25,"alert_on_rain":true}'

# 3. Trigger weather fetch
curl -X POST http://localhost:8080/api/weather/fetch

# 4. Check if alert was sent
curl http://localhost:8080/api/users/{user_id}/alerts
```

For comprehensive testing guide, see [TESTING_GUIDE.md](docs/TESTING_GUIDE.md)

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure:

- Code follows Rust style guidelines (`cargo fmt`)
- All tests pass (`cargo test`)
- No clippy warnings (`cargo clippy`)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Actix-web](https://actix.rs/) - Web framework
- [tokio-cron-scheduler](https://github.com/mvniekerk/tokio-cron-scheduler) - CRON scheduling
- [SQLx](https://github.com/launchbadge/sqlx) - Async PostgreSQL driver
- [OpenWeatherMap](https://openweathermap.org/) - Weather data API
- [Lettre](https://github.com/lettre/lettre) - Email library

## Support

- **Issues:** [GitHub Issues](https://github.com/bhaweshpanwar/weather-alert-system/issues)
- **Documentation:** [Wiki](https://github.com/bhaweshpanwar/weather-alert-system/wiki)
- **Email:** support@example.com

## 🗺️ Roadmap

### v1.1 (Planned)

- [ ] SMS alerts (Twilio integration)
- [ ] Web dashboard (React frontend)
- [ ] Docker support
- [ ] Kubernetes manifests

### v1.2 (Planned)

- [ ] Push notifications
- [ ] Weather forecasts (5-day)
- [ ] Multiple locations per user
- [ ] Mobile app (React Native)

### v2.0 (Future)

- [ ] Machine learning weather predictions
- [ ] GraphQL API
- [ ] Real-time WebSocket updates
- [ ] Advanced analytics dashboard

---

<div align="center">

**Made with ❤️ using Rust**

⭐ Star this repo if you find it useful!

[Report Bug](https://github.com/bhaweshpanwar/weather-alert-system/issues) · [Request Feature](https://github.com/bhaweshpanwar/weather-alert-system/issues) · [Documentation](https://github.com/bhaweshpanwar/weather-alert-system/wiki)

</div>
