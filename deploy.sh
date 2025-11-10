#!/bin/bash

# Weather Alert System - AWS Deployment Script
# This is a POSIX-compliant shell script for deploying to AWS EC2

set -e  # Exit on error

echo "🚀 Weather Alert System - AWS Deployment"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
APP_NAME="weather-alert-system"
APP_DIR="/home/ubuntu/${APP_NAME}"
SERVICE_NAME="weather-alert"

# Functions
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Check if running on Ubuntu/Debian
check_os() {
    print_info "Checking operating system..."
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        if [ "$ID" = "ubuntu" ] || [ "$ID" = "debian" ]; then
            print_success "Running on $PRETTY_NAME"
        else
            print_error "This script is designed for Ubuntu/Debian"
            exit 1
        fi
    fi
}

# Install system dependencies
install_dependencies() {
    print_info "Installing system dependencies..."
    
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        postgresql-client \
        curl \
        git
    
    print_success "System dependencies installed"
}

# Install Rust
install_rust() {
    print_info "Checking Rust installation..."
    
    if command -v cargo &> /dev/null; then
        print_success "Rust already installed: $(rustc --version)"
    else
        print_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
        print_success "Rust installed successfully"
    fi
}

# Setup application directory
setup_app_directory() {
    print_info "Setting up application directory..."
    
    if [ -d "$APP_DIR" ]; then
        print_info "Directory exists, backing up..."
        sudo mv "$APP_DIR" "${APP_DIR}.backup.$(date +%s)"
    fi
    
    mkdir -p "$APP_DIR"
    cd "$APP_DIR"
    
    print_success "Application directory ready: $APP_DIR"
}

# Create .env file
create_env_file() {
    print_info "Creating .env configuration..."
    
    read -p "Database URL (e.g., postgres://user:pass@host:5432/db): " DB_URL
    read -p "Weather API Key (OpenWeatherMap): " WEATHER_KEY
    read -p "SMTP Host (default: smtp.gmail.com): " SMTP_HOST
    SMTP_HOST=${SMTP_HOST:-smtp.gmail.com}
    read -p "SMTP Port (default: 587): " SMTP_PORT
    SMTP_PORT=${SMTP_PORT:-587}
    read -p "SMTP Username (email): " SMTP_USER
    read -sp "SMTP Password (app password): " SMTP_PASS
    echo
    
    cat > .env << EOF
DATABASE_URL=$DB_URL
WEATHER_API_KEY=$WEATHER_KEY
SMTP_HOST=$SMTP_HOST
SMTP_PORT=$SMTP_PORT
SMTP_USERNAME=$SMTP_USER
SMTP_PASSWORD=$SMTP_PASS
RUST_LOG=weather_alert_system=info,actix_web=info
EOF
    
    chmod 600 .env
    print_success ".env file created"
}

# Build application
build_application() {
    print_info "Building application (this may take several minutes)..."
    
    cd "$APP_DIR"
    source $HOME/.cargo/env
    
    cargo build --release
    
    print_success "Application built successfully"
}

# Initialize database
init_database() {
    print_info "Initializing database schema..."
    
    cd "$APP_DIR"
    ./target/release/${APP_NAME} init-db
    
    print_success "Database initialized"
}

# Create systemd service
create_systemd_service() {
    print_info "Creating systemd service..."
    
    sudo tee /etc/systemd/system/${SERVICE_NAME}.service > /dev/null << EOF
[Unit]
Description=Weather Alert System
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=$APP_DIR
EnvironmentFile=$APP_DIR/.env
ExecStart=$APP_DIR/target/release/${APP_NAME} serve --port 8080
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
    
    sudo systemctl daemon-reload
    print_success "Systemd service created"
}

# Start service
start_service() {
    print_info "Starting ${SERVICE_NAME} service..."
    
    sudo systemctl enable ${SERVICE_NAME}
    sudo systemctl start ${SERVICE_NAME}
    
    sleep 2
    
    if sudo systemctl is-active --quiet ${SERVICE_NAME}; then
        print_success "Service started successfully"
    else
        print_error "Service failed to start. Check logs with: sudo journalctl -u ${SERVICE_NAME}"
        exit 1
    fi
}

# Install and configure nginx
setup_nginx() {
    print_info "Do you want to setup Nginx reverse proxy? (y/n)"
    read -r setup_nginx_choice
    
    if [ "$setup_nginx_choice" = "y" ]; then
        print_info "Installing Nginx..."
        sudo apt-get install -y nginx
        
        read -p "Enter domain name (or press Enter for IP only): " DOMAIN_NAME
        
        if [ -z "$DOMAIN_NAME" ]; then
            SERVER_NAME="_"
        else
            SERVER_NAME="$DOMAIN_NAME"
        fi
        
        sudo tee /etc/nginx/sites-available/${APP_NAME} > /dev/null << EOF
server {
    listen 80;
    server_name $SERVER_NAME;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF
        
        sudo ln -sf /etc/nginx/sites-available/${APP_NAME} /etc/nginx/sites-enabled/
        sudo rm -f /etc/nginx/sites-enabled/default
        sudo nginx -t
        sudo systemctl restart nginx
        
        print_success "Nginx configured"
    fi
}

# Setup SSL with Let's Encrypt
setup_ssl() {
    print_info "Do you want to setup SSL with Let's Encrypt? (y/n)"
    read -r setup_ssl_choice
    
    if [ "$setup_ssl_choice" = "y" ]; then
        read -p "Enter your domain name: " DOMAIN_NAME
        read -p "Enter your email: " EMAIL
        
        sudo apt-get install -y certbot python3-certbot-nginx
        sudo certbot --nginx -d "$DOMAIN_NAME" --non-interactive --agree-tos -m "$EMAIL"
        
        print_success "SSL certificate installed"
    fi
}

# Print status
print_status() {
    echo ""
    echo "=========================================="
    echo "🎉 Deployment Complete!"
    echo "=========================================="
    echo ""
    echo "Service Status:"
    sudo systemctl status ${SERVICE_NAME} --no-pager
    echo ""
    echo "📍 Application URL: http://$(curl -s ifconfig.me):8080"
    echo ""
    echo "Useful Commands:"
    echo "  View logs:        sudo journalctl -u ${SERVICE_NAME} -f"
    echo "  Restart service:  sudo systemctl restart ${SERVICE_NAME}"
    echo "  Stop service:     sudo systemctl stop ${SERVICE_NAME}"
    echo "  Test email:       cd $APP_DIR && ./target/release/${APP_NAME} test-email --to your@email.com"
    echo "  Manual fetch:     cd $APP_DIR && ./target/release/${APP_NAME} fetch-weather"
    echo ""
    echo "API Endpoints:"
    echo "  Health Check:     GET  http://your-ip:8080/api/health"
    echo "  Register User:    POST http://your-ip:8080/api/users"
    echo "  Get Weather:      GET  http://your-ip:8080/api/weather/current/{city}"
    echo ""
    print_success "All done! Your weather alert system is running!"
}

# Main deployment flow
main() {
    echo ""
    check_os
    echo ""
    
    install_dependencies
    echo ""
    
    install_rust
    echo ""
    
    setup_app_directory
    echo ""
    
    print_info "Do you have the source code ready? (y/n)"
    read -r has_source
    
    if [ "$has_source" != "y" ]; then
        print_error "Please upload source code to $APP_DIR first"
        print_info "Use: scp -r ./src Cargo.toml ubuntu@your-ip:$APP_DIR/"
        exit 1
    fi
    
    create_env_file
    echo ""
    
    build_application
    echo ""
    
    init_database
    echo ""
    
    create_systemd_service
    echo ""
    
    start_service
    echo ""
    
    setup_nginx
    echo ""
    
    setup_ssl
    echo ""
    
    print_status
}

# Run main function
main