use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::activity::model::ActivityEvent;
use crate::error::AppResult;
use crate::helpers::pagination::Pagination;

pub async fn get_user_feed(
    pool: &PgPool,
    user_id: Uuid,
    page: u32,
    per_page: u32,
) -> AppResult<(Vec<ActivityEvent>, Pagination)> {
    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM activity_events WHERE user_id = $1"#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let pagination = Pagination::new(page, per_page, total.0 as u64);

    let events = sqlx::query_as::<_, ActivityEvent>(
        r#"SELECT a.*,
                  u.username, u.display_name,
                  r.name as repo_name, r.owner_type as repo_owner_type, r.owner_id as repo_owner_id
           FROM activity_events a
           LEFT JOIN users u ON a.user_id = u.id
           LEFT JOIN repositories r ON a.repository_id = r.id
           WHERE a.user_id = $1
           ORDER BY a.created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(user_id)
    .bind(per_page as i64)
    .bind(pagination.offset() as i64)
    .fetch_all(pool)
    .await?;

    Ok((events, pagination))
}

pub async fn get_dashboard_feed(
    pool: &PgPool,
    user_id: Uuid,
    page: u32,
    per_page: u32,
) -> AppResult<(Vec<ActivityEvent>, Pagination)> {
    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM activity_events a
           WHERE a.user_id = $1
              OR a.repository_id IN (
                  SELECT repository_id FROM repository_members WHERE user_id = $1
              )
              OR a.repository_id IN (
                  SELECT r.id FROM repositories r
                  JOIN group_members gm ON r.owner_id = gm.group_id AND r.owner_type = 'group'
                  WHERE gm.user_id = $1
              )"#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let pagination = Pagination::new(page, per_page, total.0 as u64);

    let events = sqlx::query_as::<_, ActivityEvent>(
        r#"SELECT a.*,
                  u.username, u.display_name,
                  r.name as repo_name, r.owner_type as repo_owner_type, r.owner_id as repo_owner_id
           FROM activity_events a
           LEFT JOIN users u ON a.user_id = u.id
           LEFT JOIN repositories r ON a.repository_id = r.id
           WHERE a.user_id = $1
              OR a.repository_id IN (
                  SELECT repository_id FROM repository_members WHERE user_id = $1
              )
              OR a.repository_id IN (
                  SELECT r2.id FROM repositories r2
                  JOIN group_members gm ON r2.owner_id = gm.group_id AND r2.owner_type = 'group'
                  WHERE gm.user_id = $1
              )
           ORDER BY a.created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(user_id)
    .bind(per_page as i64)
    .bind(pagination.offset() as i64)
    .fetch_all(pool)
    .await?;

    Ok((events, pagination))
}

pub async fn record_event(
    pool: &PgPool,
    event_type: &str,
    user_id: Uuid,
    repository_id: Option<Uuid>,
    target_type: Option<&str>,
    target_id: Option<Uuid>,
    metadata: serde_json::Value,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO activity_events (event_type, user_id, repository_id, target_type, target_id, metadata)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(event_type)
    .bind(user_id)
    .bind(repository_id)
    .bind(target_type)
    .bind(target_id)
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn record_repo_event(
    pool: &PgPool,
    event_type: &str,
    user_id: Uuid,
    repo_id: Uuid,
    repo_name: &str,
) -> AppResult<()> {
    record_event(
        pool,
        event_type,
        user_id,
        Some(repo_id),
        Some("repository"),
        Some(repo_id),
        json!({"repo_name": repo_name}),
    )
    .await
}
