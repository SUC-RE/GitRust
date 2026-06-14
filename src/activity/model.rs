use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ActivityEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub repository_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    #[sqlx(json)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    // Joined fields
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub repo_name: Option<String>,
    pub repo_owner_type: Option<String>,
    pub repo_owner_id: Option<Uuid>,
}
