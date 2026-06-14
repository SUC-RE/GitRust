use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::merge_requests::model::{MRListItem, MergeRequest};

pub async fn create_mr(
    pool: &PgPool, repo_id: Uuid, author_id: Uuid,
    title: &str, description: &str, source_branch: &str, target_branch: &str,
) -> AppResult<MergeRequest> {
    if source_branch == target_branch {
        return Err(AppError::BadRequest("Source and target branches must be different.".into()));
    }
    let number: (i32,) = sqlx::query_as(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM merge_requests WHERE repository_id = $1",
    ).bind(repo_id).fetch_one(pool).await?;

    sqlx::query_as::<_, MergeRequest>(
        r#"INSERT INTO merge_requests (repository_id, number, title, description, author_id, source_branch, target_branch)
           VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"#,
    )
    .bind(repo_id).bind(number.0).bind(title).bind(description)
    .bind(author_id).bind(source_branch).bind(target_branch)
    .fetch_one(pool).await.map_err(AppError::from)
}

pub async fn list_mrs(
    pool: &PgPool, repo_id: Uuid, state: Option<&str>, page: u32, per_page: u32,
) -> AppResult<(Vec<MRListItem>, crate::helpers::pagination::Pagination)> {
    let state_filter = state.unwrap_or("open");
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM merge_requests WHERE repository_id = $1 AND state = $2",
    ).bind(repo_id).bind(state_filter).fetch_one(pool).await?;
    let pagination = crate::helpers::pagination::Pagination::new(page, per_page, total.0 as u64);

    let mrs = sqlx::query_as::<_, MRListItem>(
        r#"SELECT mr.id, mr.number, mr.title, mr.state, mr.source_branch, mr.target_branch,
                  u.username as author_username, u.display_name as author_display_name, mr.created_at
           FROM merge_requests mr JOIN users u ON mr.author_id = u.id
           WHERE mr.repository_id = $1 AND mr.state = $2 ORDER BY mr.created_at DESC
           LIMIT $3 OFFSET $4"#,
    ).bind(repo_id).bind(state_filter).bind(per_page as i64).bind(pagination.offset() as i64)
    .fetch_all(pool).await?;
    Ok((mrs, pagination))
}

pub async fn get_mr(pool: &PgPool, repo_id: Uuid, number: i32) -> AppResult<MergeRequest> {
    sqlx::query_as::<_, MergeRequest>(
        "SELECT * FROM merge_requests WHERE repository_id = $1 AND number = $2",
    ).bind(repo_id).bind(number).fetch_optional(pool).await?
    .ok_or_else(|| AppError::NotFound("Merge request not found.".into()))
}

pub async fn close_mr(pool: &PgPool, repo_id: Uuid, number: i32) -> AppResult<()> {
    sqlx::query("UPDATE merge_requests SET state = 'closed', closed_at = now(), updated_at = now() WHERE repository_id = $1 AND number = $2")
        .bind(repo_id).bind(number).execute(pool).await?;
    Ok(())
}

pub async fn merge_mr(pool: &PgPool, repo_id: Uuid, number: i32, merged_by: Uuid) -> AppResult<()> {
    let mr = get_mr(pool, repo_id, number).await?;
    if mr.state != "open" {
        return Err(AppError::BadRequest("Merge request is not open.".into()));
    }
    sqlx::query(
        "UPDATE merge_requests SET state = 'merged', merged_at = now(), merged_by = $1, updated_at = now() WHERE repository_id = $2 AND number = $3",
    ).bind(merged_by).bind(repo_id).bind(number).execute(pool).await?;
    Ok(())
}
