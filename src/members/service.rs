use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppResult;
use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MemberRow {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub permission: String,
    pub added_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list(pool: &PgPool, repo_id: Uuid) -> AppResult<Vec<MemberRow>> {
    Ok(sqlx::query_as::<_, MemberRow>(
        r#"SELECT rm.user_id, u.username, u.display_name, rm.permission, rm.added_at
           FROM repository_members rm JOIN users u ON rm.user_id = u.id
           WHERE rm.repository_id = $1 ORDER BY rm.added_at"#,
    ).bind(repo_id).fetch_all(pool).await?)
}

pub async fn add(pool: &PgPool, repo_id: Uuid, user_id: Uuid, permission: &str) -> AppResult<()> {
    sqlx::query("INSERT INTO repository_members (repository_id, user_id, permission) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
        .bind(repo_id).bind(user_id).bind(permission).execute(pool).await?;
    Ok(())
}

pub async fn remove(pool: &PgPool, repo_id: Uuid, user_id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM repository_members WHERE repository_id = $1 AND user_id = $2")
        .bind(repo_id).bind(user_id).execute(pool).await?;
    Ok(())
}
