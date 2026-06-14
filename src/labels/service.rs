use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppResult;
use crate::issues::model::Label;

pub async fn list_labels(pool: &PgPool, repo_id: Uuid) -> AppResult<Vec<Label>> {
    Ok(sqlx::query_as::<_, Label>(
        "SELECT * FROM issue_labels WHERE repository_id = $1 ORDER BY name"
    ).bind(repo_id).fetch_all(pool).await?)
}

pub async fn create_label(
    pool: &PgPool, repo_id: Uuid, name: &str, color: &str, description: Option<&str>,
) -> AppResult<Label> {
    Ok(sqlx::query_as::<_, Label>(
        "INSERT INTO issue_labels (repository_id, name, color, description) VALUES ($1, $2, $3, $4) RETURNING *"
    ).bind(repo_id).bind(name).bind(color).bind(description).fetch_one(pool).await?)
}

pub async fn delete_label(pool: &PgPool, repo_id: Uuid, label_id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM issue_labels WHERE id = $1 AND repository_id = $2")
        .bind(label_id).bind(repo_id).execute(pool).await?;
    Ok(())
}
