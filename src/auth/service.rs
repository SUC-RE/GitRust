use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::users::model::User;
use crate::error::{AppError, AppResult};

pub struct AuthService;

impl AuthService {
    pub async fn register(
        pool: &PgPool,
        username: &str,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> AppResult<User> {
        if username.len() < 3 || username.len() > 64 {
            return Err(AppError::BadRequest("Username must be 3-64 characters.".into()));
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(AppError::BadRequest("Username can only contain letters, numbers, -, and _.".into()));
        }
        if password.len() < 8 {
            return Err(AppError::BadRequest("Password must be at least 8 characters.".into()));
        }

        if User::find_by_username(pool, username).await?.is_some() {
            return Err(AppError::Conflict("Username already taken.".into()));
        }
        if User::find_by_email(pool, email).await?.is_some() {
            return Err(AppError::Conflict("Email already registered.".into()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hashing failed: {}", e)))?
            .to_string();

        let user = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (username, email, password_hash, display_name)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(username)
        .bind(email)
        .bind(&password_hash)
        .bind(display_name)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> AppResult<User> {
        let user = User::find_by_username(pool, username)
            .await?
            .ok_or_else(|| AppError::BadRequest("Invalid username or password.".into()))?;

        if !user.is_active {
            return Err(AppError::Forbidden("Account is disabled.".into()));
        }

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid password hash: {}", e)))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::BadRequest("Invalid username or password.".into()))?;

        Ok(user)
    }

    pub async fn verify_email(pool: &PgPool, token: &str) -> AppResult<()> {
        #[derive(sqlx::FromRow)]
        struct Row {
            user_id: Uuid,
            used: bool,
            expires_at: chrono::DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            "SELECT user_id, used, expires_at FROM email_verifications WHERE token = $1",
        )
        .bind(token)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::BadRequest("Invalid verification token.".into()))?;

        if row.used {
            return Err(AppError::BadRequest("This verification link has already been used.".into()));
        }
        if row.expires_at < Utc::now() {
            return Err(AppError::BadRequest("Verification link has expired.".into()));
        }

        sqlx::query("UPDATE email_verifications SET used = true WHERE token = $1")
            .bind(token)
            .execute(pool)
            .await?;

        sqlx::query("UPDATE users SET email_verified = true WHERE id = $1")
            .bind(row.user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn create_email_verification(
        pool: &PgPool,
        user_id: Uuid,
        token: &str,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO email_verifications (user_id, token, expires_at) VALUES ($1, $2, $3)",
        )
        .bind(user_id)
        .bind(token)
        .bind(Utc::now() + chrono::Duration::hours(24))
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn store_captcha(
        pool: &PgPool,
        token: &str,
        answer: &str,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO captcha_challenges (token, answer, expires_at) VALUES ($1, $2, $3)",
        )
        .bind(token)
        .bind(answer)
        .bind(Utc::now() + chrono::Duration::minutes(10))
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn verify_captcha(pool: &PgPool, token: &str, input: &str) -> AppResult<bool> {
        #[derive(sqlx::FromRow)]
        struct Row {
            answer: String,
            expires_at: chrono::DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            "SELECT answer, expires_at FROM captcha_challenges WHERE token = $1",
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) if r.expires_at > Utc::now() => {
                sqlx::query("DELETE FROM captcha_challenges WHERE token = $1")
                    .bind(token)
                    .execute(pool)
                    .await?;
                Ok(r.answer.to_lowercase() == input.to_lowercase())
            }
            _ => Ok(false),
        }
    }
}
