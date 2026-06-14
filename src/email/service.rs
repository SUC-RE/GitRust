use lettre::{
    message::Mailbox,
    transport::smtp::authentication::Credentials,
    AsyncTransport, Message, Tokio1Executor,
};

pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
    smtp_from: Mailbox,
    base_url: String,
}

impl EmailService {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_user: String,
        smtp_pass: String,
        smtp_from: &str,
        base_url: String,
    ) -> Self {
        let from = smtp_from.parse().unwrap_or_else(|_| {
            "noreply@gitrust.local".parse().unwrap()
        });
        EmailService {
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_pass,
            smtp_from: from,
            base_url,
        }
    }

    pub async fn send_verification_email(
        &self,
        to_address: &str,
        username: &str,
        token: &str,
    ) -> Result<(), String> {
        let to: Mailbox = to_address
            .parse()
            .map_err(|e| format!("Invalid email: {}", e))?;

        let verification_url = format!("{}/auth/verify-email/{}", self.base_url, token);

        let body = format!(
            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"></head>
<body style="font-family:sans-serif;background:#0d1117;color:#e6edf3;padding:24px;">
<div style="max-width:480px;margin:0 auto;background:#161b22;border:1px solid #30363d;border-radius:12px;padding:32px;">
<h2 style="margin-top:0;">Welcome to GitRust, {}!</h2>
<p>Please verify your email address to complete your registration.</p>
<a href="{}" style="display:inline-block;background:#1f6feb;color:#fff;padding:10px 24px;border-radius:6px;text-decoration:none;font-weight:600;">Verify Email</a>
<p style="margin-top:24px;font-size:13px;color:#8b949e;">This link expires in 24 hours. If you did not create this account, please ignore this email.</p>
</div></body></html>"#,
            username, verification_url
        );

        let email = Message::builder()
            .from(self.smtp_from.clone())
            .to(to)
            .subject("Verify your GitRust email address")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| format!("Failed to build email: {}", e))?;

        if self.smtp_host.is_empty() || self.smtp_user.is_empty() {
            tracing::warn!("SMTP not configured, skipping email to {}", to_address);
            tracing::info!("Verification URL: {}", verification_url);
            return Ok(());
        }

        let creds = Credentials::new(self.smtp_user.clone(), self.smtp_pass.clone());
        let mailer = lettre::AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_host)
            .map_err(|e| e.to_string())?
            .port(self.smtp_port)
            .credentials(creds)
            .build();

        mailer.send(email).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
