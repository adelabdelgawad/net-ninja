use sqlx::SqlitePool;

use crate::crypto::EncryptionKey;
use crate::errors::{AppError, AppResult};
use crate::models::{CreateSmtpConfigRequest, SmtpConfig, SmtpConfigTestRequest, SmtpConfigTestResponse, UpdateSmtpConfigRequest, SmtpVendor};
use crate::repositories::SmtpConfigRepository;
use crate::services::ews_client::EwsClient;

pub struct SmtpConfigService;

impl SmtpConfigService {
    pub async fn get_by_id(pool: &SqlitePool, id: i32, encryption_key: Option<&EncryptionKey>) -> AppResult<SmtpConfig> {
        SmtpConfigRepository::get_by_id(pool, id, encryption_key)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("SMTP config with id {} not found", id)))
    }

    pub async fn get_all(pool: &SqlitePool, encryption_key: Option<&EncryptionKey>) -> AppResult<Vec<SmtpConfig>> {
        SmtpConfigRepository::get_all(pool, encryption_key).await
    }

    pub async fn get_default(pool: &SqlitePool, encryption_key: Option<&EncryptionKey>) -> AppResult<SmtpConfig> {
        SmtpConfigRepository::get_default(pool, encryption_key)
            .await?
            .ok_or_else(|| AppError::NotFound("No default SMTP config found".to_string()))
    }

    pub async fn create(pool: &SqlitePool, req: CreateSmtpConfigRequest, encryption_key: Option<&EncryptionKey>) -> AppResult<SmtpConfig> {
        let mut processed_req = req.clone();

        let vendor = processed_req.vendor.unwrap_or_default();
        let config = vendor.config();

        if req.port.is_none() {
            processed_req.port = Some(config.default_port);
        }

        if req.use_tls.is_none() {
            processed_req.use_tls = Some(config.default_use_tls);
        }

        // Auto-fill host for vendors with non-empty defaults
        if processed_req.host.is_empty() && !config.default_host.is_empty() {
            processed_req.host = config.default_host.to_string();
        }

        vendor.validate(&processed_req).map_err(AppError::Validation)?;

        SmtpConfigRepository::create(pool, &processed_req, encryption_key).await
    }

    pub async fn update(pool: &SqlitePool, id: i32, req: UpdateSmtpConfigRequest, encryption_key: Option<&EncryptionKey>) -> AppResult<SmtpConfig> {
        SmtpConfigRepository::update(pool, id, &req, encryption_key)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("SMTP config with id {} not found", id)))
    }

    pub async fn delete(pool: &SqlitePool, id: i32) -> AppResult<()> {
        if !SmtpConfigRepository::delete(pool, id).await? {
            return Err(AppError::NotFound(format!("SMTP config with id {} not found", id)));
        }
        Ok(())
    }

    pub async fn set_as_default(pool: &SqlitePool, id: i32, encryption_key: Option<&EncryptionKey>) -> AppResult<SmtpConfig> {
        Self::update(
            pool,
            id,
            UpdateSmtpConfigRequest {
                name: None,
                host: None,
                port: None,
                vendor: None,
                username: None,
                password: None,
                from_email: None,
                from_name: None,
                use_tls: None,
                is_default: Some(true),
                is_active: None,
            },
            encryption_key,
        )
        .await
    }

    pub async fn test_config_with_recipient(
        pool: &SqlitePool,
        id: i32,
        test_recipient: &str,
        encryption_key: Option<&EncryptionKey>,
    ) -> AppResult<SmtpConfigTestResponse> {
        let config = Self::get_by_id(pool, id, encryption_key).await?;

        // Test the SMTP configuration by sending a test email
        match Self::send_test_email(
            &config.host,
            config.port as u16,
            config.username.as_deref(),
            config.password.as_deref(),
            &config.from_email,
            config.from_name.as_deref(),
            test_recipient,
            config.use_tls,
            config.vendor,
        )
        .await
        {
            Ok(_) => Ok(SmtpConfigTestResponse {
                success: true,
                message: format!("Test email sent successfully to {}", test_recipient),
            }),
            Err(e) => Ok(SmtpConfigTestResponse {
                success: false,
                message: format!("Failed to send test email: {}", e),
            }),
        }
    }

    pub async fn test_config_without_saving(
        req: &SmtpConfigTestRequest,
    ) -> AppResult<SmtpConfigTestResponse> {
        let vendor = req.vendor.unwrap_or_default();

        // Test the SMTP configuration without saving it
        match Self::send_test_email(
            &req.host,
            req.port as u16,
            req.username.as_deref(),
            req.password.as_deref(),
            &req.sender_email,
            req.sender_name.as_deref(),
            &req.test_recipient,
            req.use_tls.unwrap_or(true),
            vendor,
        )
        .await
        {
            Ok(_) => Ok(SmtpConfigTestResponse {
                success: true,
                message: format!("Test email sent successfully to {}", req.test_recipient),
            }),
            Err(e) => Ok(SmtpConfigTestResponse {
                success: false,
                message: format!("Failed to send test email: {}", e),
            }),
        }
    }

    /// Send an email with custom subject and HTML body
    ///
    /// # Arguments
    /// * `config_id` - SMTP configuration ID to use
    /// * `to` - List of recipient email addresses
    /// * `cc` - List of CC recipient email addresses
    /// * `subject` - Email subject
    /// * `html_body` - HTML email body
    pub async fn send_email(
        pool: &SqlitePool,
        config_id: i32,
        to: &[String],
        cc: &[String],
        subject: &str,
        html_body: &str,
        encryption_key: Option<&EncryptionKey>,
    ) -> AppResult<()> {
        let config = Self::get_by_id(pool, config_id, encryption_key).await?;

        Self::send_email_with_config(
            &config.host,
            config.port as u16,
            config.username.as_deref(),
            config.password.as_deref(),
            &config.from_email,
            config.from_name.as_deref(),
            to,
            cc,
            subject,
            html_body,
            config.use_tls,
            config.vendor,
        )
        .await
        .map_err(|e| AppError::Email(format!("Failed to send email: {}", e)))
    }

    async fn send_email_with_config(
        host: &str,
        port: u16,
        username: Option<&str>,
        password: Option<&str>,
        from_email: &str,
        from_name: Option<&str>,
        to: &[String],
        cc: &[String],
        subject: &str,
        html_body: &str,
        use_tls: bool,
        vendor: SmtpVendor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Route Exchange through EWS instead of SMTP
        if vendor == SmtpVendor::Exchange {
            let username = username.ok_or("Exchange requires username")?;
            let password = password.ok_or("Exchange requires password")?;
            let from_name_str = from_name.unwrap_or("");
            let to_str = to.join(";");
            let cc_str = cc.join(";");

            return EwsClient::send_email(
                host,
                username,
                password,
                from_email,
                from_name_str,
                &to_str,
                &cc_str,
                subject,
                html_body,
            )
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }

        // Use SMTP for non-Exchange vendors
        use lettre::{
            message::header::ContentType, Message, Transport,
        };

        if to.is_empty() {
            return Err("At least one recipient is required".into());
        }

        // Build the from address with optional name
        let from_address = if let Some(name) = from_name {
            format!("{} <{}>", name, from_email)
        } else {
            from_email.to_string()
        };

        // Build message
        let mut message_builder = Message::builder()
            .from(from_address.parse()?)
            .subject(subject);

        // Add TO recipients
        for to_addr in to {
            message_builder = message_builder.to(to_addr.parse()?);
        }

        // Add CC recipients
        for cc_addr in cc {
            message_builder = message_builder.cc(cc_addr.parse()?);
        }

        let email = message_builder
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())?;

        // Build the transport
        let mailer = Self::build_transport(host, port, username, password, use_tls)?;

        // Send the email
        tracing::info!(
            from = %from_email,
            to_count = to.len(),
            cc_count = cc.len(),
            subject = %subject,
            "[SmtpConfigService] Sending email to {} recipients (CC: {})", to.len(), cc.len()
        );
        match mailer.send(&email) {
            Ok(_) => {
                tracing::info!(
                    to_count = to.len(),
                    "[SmtpConfigService] Email sent successfully to {} recipients", to.len()
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "[SmtpConfigService] Failed to send email: {}", e
                );
                Err(Box::new(e))
            }
        }
    }

    async fn send_test_email(
        host: &str,
        port: u16,
        username: Option<&str>,
        password: Option<&str>,
        from_email: &str,
        from_name: Option<&str>,
        to_email: &str,
        use_tls: bool,
        vendor: SmtpVendor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Route Exchange through EWS instead of SMTP
        if vendor == SmtpVendor::Exchange {
            let username = username.ok_or("Exchange requires username")?;
            let password = password.ok_or("Exchange requires password")?;
            let from_name_str = from_name.unwrap_or("");

            return EwsClient::send_test_email(
                host,
                username,
                password,
                from_email,
                from_name_str,
                to_email,
            )
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }

        // Use SMTP for non-Exchange vendors
        use lettre::{
            message::header::ContentType, Message, Transport,
        };

        // Log: Building email payload
        tracing::info!(
            host = %host,
            port = %port,
            from = %from_email,
            to = %to_email,
            use_tls = %use_tls,
            "[SmtpConfigService] Building test email"
        );

        // Build the from address with optional name
        let from_address = if let Some(name) = from_name {
            format!("{} <{}>", name, from_email)
        } else {
            from_email.to_string()
        };

        let email = Message::builder()
            .from(from_address.parse()?)
            .to(to_email.parse()?)
            .subject("NetNinja SMTP Test")
            .header(ContentType::TEXT_PLAIN)
            .body("This is a test email from NetNinja to verify your SMTP configuration.".to_string())?;

        // Build the transport
        let mailer = Self::build_transport(host, port, username, password, use_tls)?;

        // Log: Before send attempt
        tracing::info!(
            to = %to_email,
            host = %host,
            port = %port,
            "[SmtpConfigService] Sending test email via {}:{port}", host
        );

        // Send the email (synchronous in lettre 0.11.x)
        match mailer.send(&email) {
            Ok(_) => {
                tracing::info!(
                    to = %to_email,
                    "[SmtpConfigService] Test email sent successfully to '{}'", to_email
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    to = %to_email,
                    error = %e,
                    "[SmtpConfigService] Failed to send test email to '{}': {}", to_email, e
                );
                Err(Box::new(e) as Box<dyn std::error::Error>)
            }
        }
    }

    fn build_transport(
        host: &str,
        port: u16,
        username: Option<&str>,
        password: Option<&str>,
        use_tls: bool,
    ) -> Result<lettre::SmtpTransport, Box<dyn std::error::Error>> {
        use lettre::transport::smtp::authentication::{Credentials, Mechanism};
        use lettre::SmtpTransport;

        // Build the transport based on port and TLS settings
        // Port 465: Implicit TLS (SmtpTransport::relay)
        // Port 587: STARTTLS (SmtpTransport::starttls_relay)
        // Port 25 or no TLS: Plain (SmtpTransport::builder_dangerous)
        let mailer = if let (Some(user), Some(pass)) = (username, password) {
            let creds = Credentials::new(user.to_string(), pass.to_string());
            if use_tls {
                if port == 465 {
                    // Implicit TLS (SSL) - try PLAIN auth first
                    SmtpTransport::relay(host)?
                        .port(port)
                        .credentials(creds.clone())
                        .authentication(vec![Mechanism::Plain, Mechanism::Login])
                        .build()
                } else {
                    // STARTTLS (port 587 or other)
                    SmtpTransport::starttls_relay(host)?
                        .port(port)
                        .credentials(creds.clone())
                        .authentication(vec![Mechanism::Plain, Mechanism::Login])
                        .build()
                }
            } else {
                // Plain connection (no encryption)
                SmtpTransport::builder_dangerous(host)
                    .port(port)
                    .credentials(creds.clone())
                    .authentication(vec![Mechanism::Plain, Mechanism::Login])
                    .build()
            }
        } else {
            // No authentication
            if use_tls {
                if port == 465 {
                    SmtpTransport::relay(host)?.port(port).build()
                } else {
                    SmtpTransport::starttls_relay(host)?.port(port).build()
                }
            } else {
                SmtpTransport::builder_dangerous(host).port(port).build()
            }
        };

        Ok(mailer)
    }
}
