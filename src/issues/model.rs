use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Issue {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub author_id: Uuid,
    pub state: String,
    pub milestone_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IssueListItem {
    pub id: Uuid,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub author_username: String,
    pub author_display_name: String,
    pub created_at: DateTime<Utc>,
    pub label_count: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct IssueDetail {
    pub id: Uuid,
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub description_html: Option<String>,
    pub state: String,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub labels: Vec<LabelInfo>,
    pub milestone_title: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LabelInfo {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Label {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
