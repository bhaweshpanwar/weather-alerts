use crate::error::AppError;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use log::info;

#[derive(Clone)]
pub struct EmailClient {
    smtp_transport: std::sync::Arc<SmtpTransport>,
    from_email: String,
}

impl EmailClient {
    pub fn new(
        smtp_host: &str,
        smtp_port: u16,
        username: &str,
        password: &str,
    ) -> Result<Self, AppError> {
        info!("📧 Initializing email client: {}:{}", smtp_host, smtp_port);

        let creds = Credentials::new(username.to_string(), password.to_string());

        let transport = SmtpTransport::relay(smtp_host)
            .map_err(|e| AppError::Email(format!("SMTP relay error: {}", e)))?
            .port(smtp_port)
            .credentials(creds)
            .build();

        Ok(Self {
            // Wrap in Arc for cloning
            smtp_transport: std::sync::Arc::new(transport),
            from_email: username.to_string(),
        })
    }

    pub async fn send_weather_alert(
        &self,
        to: &str,
        city: &str,
        alert_message: &str,
    ) -> Result<(), AppError> {
        let subject = format!("⚠️ Weather Alert for {}", city);
        
        let body = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
                              color: white; padding: 30px; border-radius: 10px 10px 0 0; }}
                    .content {{ background: #f4f4f4; padding: 30px; border-radius: 0 0 10px 10px; }}
                    .alert-box {{ background: #fff3cd; border-left: 4px solid #ffc107; 
                                  padding: 15px; margin: 20px 0; border-radius: 5px; }}
                    .footer {{ text-align: center; margin-top: 20px; color: #666; font-size: 12px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>🌤️ Weather Alert System</h1>
                        <p>Your personalized weather notification</p>
                    </div>
                    <div class="content">
                        <h2>Alert for {}</h2>
                        <div class="alert-box">
                            <strong>Alert Message:</strong><br/>
                            {}
                        </div>
                        <p>This alert was triggered based on your weather preferences.</p>
                        <p><strong>What to do?</strong></p>
                        <ul>
                            <li>Check the current conditions</li>
                            <li>Plan accordingly for your day</li>
                            <li>Update your preferences if needed</li>
                        </ul>
                    </div>
                    <div class="footer">
                        <p>Weather Alert System - Powered by OpenWeatherMap</p>
                        <p>To update your preferences, visit your dashboard</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            city, alert_message
        );

        self.send_email(to, &subject, &body).await
    }

    pub async fn send_welcome_email(&self, to: &str, city: &str) -> Result<(), AppError> {
        let subject = "Welcome to Weather Alert System! 🌤️";
        
        let body = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
                              color: white; padding: 30px; border-radius: 10px 10px 0 0; text-align: center; }}
                    .content {{ background: #f4f4f4; padding: 30px; border-radius: 0 0 10px 10px; }}
                    .button {{ display: inline-block; padding: 12px 30px; background: #667eea; 
                              color: white; text-decoration: none; border-radius: 5px; margin-top: 20px; }}
                    .footer {{ text-align: center; margin-top: 20px; color: #666; font-size: 12px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>🎉 Welcome!</h1>
                        <p>You're now registered for weather alerts</p>
                    </div>
                    <div class="content">
                        <h2>Hi there! 👋</h2>
                        <p>Thank you for registering with Weather Alert System!</p>
                        <p><strong>Your Location:</strong> {}</p>
                        <p>We'll monitor the weather in your area and send you alerts based on your preferences.</p>
                        
                        <h3>What's Next?</h3>
                        <ul>
                            <li>Set your temperature thresholds (min/max)</li>
                            <li>Choose weather conditions to be alerted about (rain, snow, storms)</li>
                            <li>Receive automatic alerts every 2 hours</li>
                        </ul>
                        
                        <p>Our CRON job runs every 2 hours to check weather conditions and send alerts.</p>
                    </div>
                    <div class="footer">
                        <p>Weather Alert System - Stay informed, stay prepared</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            city
        );

        self.send_email(to, subject, &body).await
    }

    pub async fn send_test_email(&self, to: &str, subject: &str) -> Result<(), AppError> {
        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; padding: 20px;">
                <h2>✅ Email Configuration Test</h2>
                <p>If you're reading this, your email configuration is working correctly!</p>
                <p><strong>Test Details:</strong></p>
                <ul>
                    <li>Recipient: {}</li>
                    <li>Subject: {}</li>
                    <li>Time: {}</li>
                </ul>
                <p>Your Weather Alert System is ready to send notifications.</p>
            </body>
            </html>
            "#,
            to,
            subject,
            chrono::Utc::now().to_rfc3339()
        );

        self.send_email(to, subject, &body).await
    }

    async fn send_email(&self, to: &str, subject: &str, html_body: &str) -> Result<(), AppError> {
        let email = Message::builder()
            .from(
                self.from_email
                    .parse()
                    .map_err(|e| AppError::Email(format!("Invalid from address: {}", e)))?,
            )
            .to(to
                .parse()
                .map_err(|e| AppError::Email(format!("Invalid to address: {}", e)))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
            .map_err(|e| AppError::Email(format!("Failed to build email: {}", e)))?;

        // Clone the transport Arc and message to move into the blocking task
        let transport = self.smtp_transport.clone();
        let email_to_send = email.clone();

        // Use spawn_blocking for synchronous I/O in an async function
        tokio::task::spawn_blocking(move || {
            transport.send(&email_to_send)
        })
        .await
        .map_err(|e| AppError::Email(format!("Task spawn error: {}", e)))? // Handle task join error
        .map_err(|e| AppError::Email(format!("Failed to send email: {}", e)))?; // Handle email sending error

        info!("✅ Email sent to: {}", to);
        Ok(())
    }
}