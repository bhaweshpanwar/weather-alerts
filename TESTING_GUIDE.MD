# 🧪 Weather Alert System - Testing Guide

## 📋 Quick Test Checklist

```bash
# Copy this checklist and check off as you test!
[ ] Database connection works
[ ] Weather API fetches data
[ ] Email sends successfully
[ ] User registration works
[ ] Preferences update works
[ ] CRON job runs automatically
[ ] Alerts trigger correctly
[ ] API endpoints respond
[ ] Service restarts automatically
[ ] Logs are readable
```

---

## 1️⃣ Pre-Deployment Tests (Local)

### Test 1: Database Connection

```bash
# Start PostgreSQL
sudo systemctl start postgresql

# Test connection
psql -h localhost -U weather_user -d weather_alerts -c "SELECT 1;"

# Expected output: 1
```

### Test 2: Build Project

```bash
cd weather-alert-system

# Clean build
cargo clean
cargo build --release

# Should complete without errors
# Binary location: ./target/release/weather-alert-system
```

### Test 3: Initialize Database

```bash
cargo run -- init-db

# Expected output:
# 🔌 Connecting to PostgreSQL database...
# ✅ Database connection established
# 📋 Creating database schema...
# ✅ Database schema created successfully
```

### Test 4: Email Configuration

```bash
# Test email (replace with your email)
cargo run -- test-email --to your_email@gmail.com

# Expected: You receive an email within 30 seconds
# If fails, check:
# - SMTP credentials in .env
# - Using App Password (not regular password)
# - Gmail 2FA enabled
```

### Test 5: Weather API

```bash
# Test manually with curl
curl "https://api.openweathermap.org/data/2.5/weather?q=London,GB&appid=YOUR_API_KEY&units=metric"

# Should return JSON with weather data
# If 401 error: API key is invalid
# If 429 error: Rate limit exceeded (wait a minute)
```

---

## 2️⃣ Functional Tests (API Endpoints)

### Start the Server

```bash
cargo run -- serve --port 8080

# Server should start on http://localhost:8080
```

### Test 1: Health Check

```bash
curl http://localhost:8080/api/health

# Expected response:
{
  "status": "healthy",
  "service": "Weather Alert System",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Test 2: Register User

```bash
# Register user in New York
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test1@example.com",
    "city": "New York",
    "country": "US"
  }'

# Expected response:
{
  "success": true,
  "data": {
    "id": "uuid-here",
    "email": "test1@example.com",
    "city": "New York",
    "country": "US",
    "created_at": "..."
  },
  "message": "User registered successfully. Welcome email sent!"
}

# ✅ Check: You receive welcome email
# ❌ If duplicate email error: Use different email
```

### Test 3: Register Multiple Users

```bash
# User 2 - London
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test2@example.com",
    "city": "London",
    "country": "GB"
  }'

# User 3 - Dubai (hot city for testing)
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test3@example.com",
    "city": "Dubai",
    "country": "AE"
  }'

# User 4 - Mumbai (for rain testing)
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test4@example.com",
    "city": "Mumbai",
    "country": "IN"
  }'
```

### Test 4: Get All Users

```bash
curl http://localhost:8080/api/users

# Expected: Array of all registered users
```

### Test 5: Get Single User

```bash
# Use user_id from registration response
curl http://localhost:8080/api/users/{user_id}

# Expected: User details with preferences
```

### Test 6: Update Preferences

```bash
# Set temperature alerts
curl -X PUT http://localhost:8080/api/users/{user_id}/preferences \
  -H "Content-Type: application/json" \
  -d '{
    "min_temp": 5,
    "max_temp": 30,
    "alert_on_rain": true,
    "alert_on_snow": false,
    "alert_on_storm": true
  }'

# Expected response: Updated preferences object
```

### Test 7: Manual Weather Fetch

```bash
curl -X POST http://localhost:8080/api/weather/fetch

# Expected response:
{
  "success": false,
  "data": null,
  "message": "Weather fetch started in background"
}

# ✅ Check server logs to see weather being fetched
# tail -f logs or check terminal output
```

### Test 8: Get Current Weather

```bash
curl http://localhost:8080/api/weather/current/London

# Expected: Latest weather data for London
# If empty: Run manual fetch first
```

### Test 9: Get Weather History

```bash
curl "http://localhost:8080/api/weather/history/London?limit=24"

# Expected: Array of weather records (up to 24)
```

### Test 10: Get User Alerts

```bash
curl http://localhost:8080/api/users/{user_id}/alerts?limit=10

# Expected: Array of alerts sent to user
```

---

## 3️⃣ CRON Job Tests

### Test 1: Verify CRON Schedule

```bash
# Check server logs for CRON setup message
# Look for: "✅ CRON job scheduled: Weather fetch every 2 hours"

# Or run list-jobs command
cargo run -- list-jobs
```

### Test 2: Change to Every Minute (Testing)

Edit `src/main.rs`, find this line:

```rust
let job = Job::new_async("0 0 */2 * * *", move |_uuid, _l| {
```

Change to:

```rust
let job = Job::new_async("0 * * * * *", move |_uuid, _l| {
```

Rebuild and restart:

```bash
cargo build --release
cargo run -- serve --port 8080
```

**Watch logs** - should see weather fetch every minute:

```
🌤️  CRON Job: Starting weather fetch...
📍 Found 4 unique cities to fetch
🌍 Fetching weather for London, GB
✅ Weather fetched: London - 12.5°C, Clouds
💾 Stored weather: London - 12.5°C, Clouds
...
```

### Test 3: Trigger Alert

**Setup:**

1. Register user in Dubai (hot city)
2. Set max_temp to 20°C (will definitely trigger)
3. Wait for CRON or run manual fetch

```bash
# 1. Register
USER_RESPONSE=$(curl -s -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alert_test@example.com",
    "city": "Dubai",
    "country": "AE"
  }')

USER_ID=$(echo $USER_RESPONSE | jq -r '.data.id')

# 2. Set low max_temp
curl -X PUT http://localhost:8080/api/users/$USER_ID/preferences \
  -H "Content-Type: application/json" \
  -d '{
    "max_temp": 20
  }'

# 3. Trigger fetch
curl -X POST http://localhost:8080/api/weather/fetch

# ✅ Check: Alert email received!
# Email subject: "⚠️ Weather Alert for Dubai"
```

### Test 4: Multiple Alert Conditions

```bash
# Test rain alerts
# 1. Find a city with rain (check weather forecast)
# 2. Register user there with alert_on_rain: true
# 3. Trigger fetch

# Test storm alerts
# Similar process with alert_on_storm: true
```

---

## 4️⃣ Database Tests

### Test 1: Verify Schema

```bash
psql -h localhost -U weather_user -d weather_alerts

# In psql:
\dt

# Expected tables:
# users
# user_preferences
# weather_data
# alert_logs

# Check table structure:
\d users
\d user_preferences
\d weather_data
\d alert_logs
```

### Test 2: Query Data

```sql
-- Count users
SELECT COUNT(*) FROM users;

-- View all users
SELECT * FROM users;

-- View preferences
SELECT u.email, p.min_temp, p.max_temp, p.alert_on_rain
FROM users u
JOIN user_preferences p ON u.id = p.user_id;

-- Latest weather for each city
SELECT DISTINCT ON (city)
    city, temperature, conditions, fetched_at
FROM weather_data
ORDER BY city, fetched_at DESC;

-- Recent alerts
SELECT u.email, a.alert_type, a.message, a.sent_at
FROM alert_logs a
JOIN users u ON a.user_id = u.id
ORDER BY a.sent_at DESC
LIMIT 10;

-- Weather history for London
SELECT temperature, conditions, fetched_at
FROM weather_data
WHERE city = 'London'
ORDER BY fetched_at DESC
LIMIT 10;
```

### Test 3: Data Integrity

```sql
-- Check for orphaned preferences (should be 0)
SELECT COUNT(*)
FROM user_preferences p
LEFT JOIN users u ON p.user_id = u.id
WHERE u.id IS NULL;

-- Check for duplicate emails (should be 0)
SELECT email, COUNT(*)
FROM users
GROUP BY email
HAVING COUNT(*) > 1;

-- Verify cascade delete
DELETE FROM users WHERE email = 'test_delete@example.com';
-- Preferences and alerts should also be deleted
```

---

## 5️⃣ Error Handling Tests

### Test 1: Invalid Email

```bash
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "not-an-email",
    "city": "London",
    "country": "GB"
  }'

# Expected: 400 Bad Request with validation error
```

### Test 2: Duplicate Email

```bash
# Register same email twice
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "duplicate@example.com",
    "city": "London",
    "country": "GB"
  }'

# Second attempt:
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "duplicate@example.com",
    "city": "Paris",
    "country": "FR"
  }'

# Expected: 409 Conflict error
```

### Test 3: Non-existent User

```bash
curl http://localhost:8080/api/users/00000000-0000-0000-0000-000000000000

# Expected: 404 Not Found
```

### Test 4: Invalid City

```bash
curl http://localhost:8080/api/weather/current/InvalidCityName123

# Expected: 404 Not Found or empty result
```

### Test 5: Database Disconnection

```bash
# Stop PostgreSQL
sudo systemctl stop postgresql

# Try API call
curl http://localhost:8080/api/users

# Expected: 500 Internal Server Error

# Start PostgreSQL again
sudo systemctl start postgresql
```

---

## 6️⃣ Load Testing

### Test 1: Concurrent User Registration

```bash
# Install Apache Bench
sudo apt-get install apache2-utils

# Create test payload
echo '{
  "email": "load_test@example.com",
  "city": "London",
  "country": "GB"
}' > user.json

# Run load test (100 requests, 10 concurrent)
ab -n 100 -c 10 -p user.json -T application/json \
  http://localhost:8080/api/users
```

### Test 2: Multiple Cities

```bash
# Register users in 20 different cities
# Then trigger weather fetch
# Measure time taken

# Cities list
CITIES=(
  "London,GB" "Paris,FR" "Berlin,DE" "Madrid,ES" "Rome,IT"
  "New York,US" "Los Angeles,US" "Chicago,US" "Houston,US" "Miami,US"
  "Tokyo,JP" "Beijing,CN" "Mumbai,IN" "Dubai,AE" "Sydney,AU"
  "Moscow,RU" "São Paulo,BR" "Mexico City,MX" "Cairo,EG" "Lagos,NG"
)

for city in "${CITIES[@]}"; do
  IFS=',' read -r city_name country <<< "$city"
  echo "Registering user in $city_name..."

  curl -s -X POST http://localhost:8080/api/users \
    -H "Content-Type: application/json" \
    -d "{
      \"email\": \"test_${city_name}@example.com\",
      \"city\": \"${city_name}\",
      \"country\": \"${country}\"
    }" > /dev/null
done

# Trigger fetch and time it
time curl -X POST http://localhost:8080/api/weather/fetch
```

---

## 7️⃣ AWS Deployment Tests

### Test 1: SSH Connection

```bash
ssh -i your-key.pem ubuntu@your-ec2-ip

# Should connect successfully
```

### Test 2: Service Status

```bash
sudo systemctl status weather-alert

# Expected: active (running)
```

### Test 3: Port Accessibility

```bash
# From your local machine
curl http://your-ec2-ip:8080/api/health

# Expected: JSON response with "healthy"
```

### Test 4: RDS Connection

```bash
# On EC2 instance
psql -h your-rds-endpoint.rds.amazonaws.com \
     -U postgres \
     -d weather_alerts \
     -c "SELECT COUNT(*) FROM users;"

# Should show user count
```

### Test 5: Logs

```bash
# View real-time logs
sudo journalctl -u weather-alert -f

# Check for errors
sudo journalctl -u weather-alert -p err

# Last 100 lines
sudo journalctl -u weather-alert -n 100
```

### Test 6: Auto-restart

```bash
# Kill the process
sudo systemctl restart weather-alert

# Check if it restarts
sudo systemctl status weather-alert

# Should show: active (running)
```

---

## 8️⃣ Integration Test Script

Create `test_all.sh`:

```bash
#!/bin/bash

BASE_URL="http://localhost:8080"
TEST_EMAIL="integration_test@example.com"

echo "🧪 Running Integration Tests..."

# Test 1: Health Check
echo "Test 1: Health Check"
HEALTH=$(curl -s $BASE_URL/api/health | jq -r '.status')
if [ "$HEALTH" = "healthy" ]; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
    exit 1
fi

# Test 2: User Registration
echo "Test 2: User Registration"
USER_RESPONSE=$(curl -s -X POST $BASE_URL/api/users \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$TEST_EMAIL\",
    \"city\": \"London\",
    \"country\": \"GB\"
  }")

USER_ID=$(echo $USER_RESPONSE | jq -r '.data.id')
if [ "$USER_ID" != "null" ]; then
    echo "✅ PASS - User ID: $USER_ID"
else
    echo "❌ FAIL"
    exit 1
fi

# Test 3: Update Preferences
echo "Test 3: Update Preferences"
PREF_RESPONSE=$(curl -s -X PUT $BASE_URL/api/users/$USER_ID/preferences \
  -H "Content-Type: application/json" \
  -d '{
    "max_temp": 30,
    "alert_on_rain": true
  }')

if echo $PREF_RESPONSE | jq -e '.success' > /dev/null; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
    exit 1
fi

# Test 4: Weather Fetch
echo "Test 4: Trigger Weather Fetch"
curl -s -X POST $BASE_URL/api/weather/fetch > /dev/null
sleep 5  # Wait for background job

# Test 5: Get Weather
echo "Test 5: Get Weather Data"
WEATHER=$(curl -s $BASE_URL/api/weather/current/London | jq -r '.data.temperature')
if [ "$WEATHER" != "null" ]; then
    echo "✅ PASS - Temperature: ${WEATHER}°C"
else
    echo "❌ FAIL"
    exit 1
fi

echo ""
echo "🎉 All Integration Tests Passed!"
```

Run it:

```bash
chmod +x test_all.sh
./test_all.sh
```

---

## 📊 Success Criteria

Your system is working if:

✅ **Email Test**: Receive test email within 30 seconds  
✅ **Registration**: Can create users via API  
✅ **Weather Fetch**: Data appears in database after fetch  
✅ **Alerts**: Receive email when conditions match preferences  
✅ **CRON**: Logs show automatic fetch every 2 hours  
✅ **Persistence**: Data survives server restart  
✅ **Error Handling**: Invalid requests return proper errors  
✅ **AWS**: Service runs on EC2, connects to RDS

---

## 🐛 Common Issues & Solutions

**Issue**: Email not sending  
**Solution**: Check App Password, enable 2FA, check firewall on port 587

**Issue**: Weather API 401  
**Solution**: Verify API key, check OpenWeatherMap account status

**Issue**: CRON not running  
**Solution**: Check logs for errors, verify scheduler started

**Issue**: Database connection refused  
**Solution**: Check PostgreSQL running, verify credentials in .env

**Issue**: Port 8080 in use  
**Solution**: `lsof -i :8080` to find process, kill it or use different port

---

**🎯 READY TO TEST? START FROM THE TOP AND CHECK EVERYTHING!** 🚀
