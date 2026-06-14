use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Milestone {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub state: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MilestoneWithProgress {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub state: String,
    pub open_issues: Option<i64>,
    pub closed_issues: Option<i64>,
    pub created_at: DateTime<Utc>,
}
