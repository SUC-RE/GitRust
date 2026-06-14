use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::users::model::{User, UserInfo};

pub async fn get_user_profile(pool: &PgPool, username: &str) -> AppResult<UserInfo> {
    let user = User::find_by_username(pool, username)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found.".into()))?;
    Ok(UserInfo::from(user))
}

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> AppResult<UserInfo> {
    let user = User::find_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found.".into()))?;
    Ok(UserInfo::from(user))
}

pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    display_name: &str,
    bio: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE users SET display_name = $1, bio = $2, updated_at = now() WHERE id = $3",
    )
    .bind(display_name)
    .bind(bio)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn change_password(
    pool: &PgPool,
    user_id: Uuid,
    current_password: &str,
    new_password: &str,
) -> AppResult<()> {
    use argon2::{
        password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
        Argon2,
    };
    use argon2::password_hash::rand_core::OsRng;

    let user = User::find_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found.".into()))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid hash: {}", e)))?;

    Argon2::default()
        .verify_password(current_password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::BadRequest("Current password is incorrect.".into()))?;

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Hash failed: {}", e)))?
        .to_string();

    sqlx::query("UPDATE users SET password_hash = $1, updated_at = now() WHERE id = $2")
        .bind(&new_hash)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_user_repos_count(pool: &PgPool, user_id: Uuid) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM repositories WHERE owner_type = 'user' AND owner_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn get_user_groups_count(pool: &PgPool, user_id: Uuid) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM group_members WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn list_ssh_keys(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<SshKeyInfo>> {
    let keys = sqlx::query_as::<_, SshKeyRow>(
        "SELECT id, title, fingerprint, created_at FROM ssh_keys WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(keys.into_iter().map(|k| SshKeyInfo {
        id: k.id,
        title: k.title,
        fingerprint: k.fingerprint,
        created_at: k.created_at,
    }).collect())
}

pub async fn add_ssh_key(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    public_key: &str,
) -> AppResult<SshKeyInfo> {
    let fingerprint = format!("SHA256:{}", &public_key.chars().take(32).collect::<String>());

    let row = sqlx::query_as::<_, SshKeyRow>(
        "INSERT INTO ssh_keys (user_id, title, public_key, fingerprint) VALUES ($1, $2, $3, $4)
         RETURNING id, title, fingerprint, created_at",
    )
    .bind(user_id)
    .bind(title)
    .bind(public_key)
    .bind(&fingerprint)
    .fetch_one(pool)
    .await?;

    Ok(SshKeyInfo {
        id: row.id,
        title: row.title,
        fingerprint: row.fingerprint,
        created_at: row.created_at,
    })
}

pub async fn delete_ssh_key(pool: &PgPool, user_id: Uuid, key_id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM ssh_keys WHERE id = $1 AND user_id = $2")
        .bind(key_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("SSH key not found.".into()));
    }
    Ok(())
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
struct SshKeyRow {
    id: Uuid,
    title: String,
    fingerprint: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct SshKeyInfo {
    pub id: Uuid,
    pub title: String,
    pub fingerprint: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
