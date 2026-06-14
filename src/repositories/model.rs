use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Repository {
    pub id: Uuid,
    pub owner_type: String,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub is_private: bool,
    pub is_archived: bool,
    pub is_template: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Repository {
    pub async fn find_by_owner_and_name(
        pool: &PgPool,
        owner_type: &str,
        owner_id: Uuid,
        name: &str,
    ) -> Result<Option<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories WHERE owner_type = $1 AND owner_id = $2 AND name = $3",
        )
        .bind(owner_type)
        .bind(owner_id)
        .bind(name)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }
}

#[derive(Serialize)]
pub struct RepoInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_name: String,
    pub owner_type: String,
    pub default_branch: String,
    pub is_private: bool,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
}
