use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MergeRequest {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub author_id: Uuid,
    pub source_branch: String,
    pub target_branch: String,
    pub state: String,
    pub merge_status: Option<String>,
    pub merge_commit_sha: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub merged_by: Option<Uuid>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MRListItem {
    pub id: Uuid,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub author_username: String,
    pub author_display_name: String,
    pub created_at: DateTime<Utc>,
}
