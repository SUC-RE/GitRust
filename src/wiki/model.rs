use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WikiPage {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub author_id: Uuid,
    pub revision: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WikiPageWithAuthor {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub revision: i32,
    pub author_username: String,
    pub author_display_name: String,
    pub updated_at: DateTime<Utc>,
}
