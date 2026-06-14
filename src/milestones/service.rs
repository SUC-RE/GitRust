use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppResult;
use crate::milestones::model::{Milestone, MilestoneWithProgress};

pub async fn list(pool: &PgPool, repo_id: Uuid) -> AppResult<Vec<MilestoneWithProgress>> {
    Ok(sqlx::query_as::<_, MilestoneWithProgress>(
        r#"SELECT m.*,
           (SELECT COUNT(*) FROM issues WHERE milestone_id = m.id AND state = 'open') as open_issues,
           (SELECT COUNT(*) FROM issues WHERE milestone_id = m.id AND state = 'closed') as closed_issues
           FROM milestones m WHERE m.repository_id = $1 ORDER BY m.created_at DESC"#,
    ).bind(repo_id).fetch_all(pool).await?)
}

pub async fn create(pool: &PgPool, repo_id: Uuid, title: &str, description: Option<&str>, due_date: Option<chrono::NaiveDate>) -> AppResult<Milestone> {
    Ok(sqlx::query_as::<_, Milestone>(
        "INSERT INTO milestones (repository_id, title, description, due_date) VALUES ($1, $2, $3, $4) RETURNING *"
    ).bind(repo_id).bind(title).bind(description).bind(due_date).fetch_one(pool).await?)
}

pub async fn get(pool: &PgPool, repo_id: Uuid, milestone_id: Uuid) -> AppResult<Milestone> {
    Ok(sqlx::query_as::<_, Milestone>(
        "SELECT * FROM milestones WHERE id = $1 AND repository_id = $2"
    ).bind(milestone_id).bind(repo_id).fetch_one(pool).await?)
}
