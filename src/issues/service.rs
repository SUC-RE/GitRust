use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::issues::model::{Issue, IssueDetail, IssueListItem, LabelInfo};

pub async fn create_issue(
    pool: &PgPool,
    repo_id: Uuid,
    author_id: Uuid,
    title: &str,
    description: &str,
) -> AppResult<Issue> {
    let number: (i32,) = sqlx::query_as(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM issues WHERE repository_id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    sqlx::query_as::<_, Issue>(
        r#"INSERT INTO issues (repository_id, number, title, description, author_id)
           VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
    )
    .bind(repo_id)
    .bind(number.0)
    .bind(title)
    .bind(description)
    .bind(author_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn list_issues(
    pool: &PgPool,
    repo_id: Uuid,
    state: Option<&str>,
    page: u32,
    per_page: u32,
) -> AppResult<(Vec<IssueListItem>, crate::helpers::pagination::Pagination)> {
    let state_filter = state.unwrap_or("open");

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM issues WHERE repository_id = $1 AND state = $2",
    )
    .bind(repo_id)
    .bind(state_filter)
    .fetch_one(pool)
    .await?;

    let pagination = crate::helpers::pagination::Pagination::new(page, per_page, total.0 as u64);

    let issues = sqlx::query_as::<_, IssueListItem>(
        r#"SELECT i.id, i.number, i.title, i.state,
                  u.username as author_username, u.display_name as author_display_name,
                  i.created_at,
                  (SELECT COUNT(*) FROM issue_label_assignments WHERE issue_id = i.id) as label_count
           FROM issues i
           JOIN users u ON i.author_id = u.id
           WHERE i.repository_id = $1 AND i.state = $2
           ORDER BY i.created_at DESC
           LIMIT $3 OFFSET $4"#,
    )
    .bind(repo_id)
    .bind(state_filter)
    .bind(per_page as i64)
    .bind(pagination.offset() as i64)
    .fetch_all(pool)
    .await?;

    Ok((issues, pagination))
}

pub async fn get_issue(
    pool: &PgPool,
    repo_id: Uuid,
    number: i32,
) -> AppResult<IssueDetail> {
    let issue = sqlx::query_as::<_, Issue>(
        "SELECT * FROM issues WHERE repository_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Issue not found.".into()))?;

    #[derive(sqlx::FromRow)]
    struct AuthorRow {
        username: String,
        display_name: String,
    }
    let author = sqlx::query_as::<_, AuthorRow>(
        "SELECT username, display_name FROM users WHERE id = $1",
    )
    .bind(issue.author_id)
    .fetch_one(pool)
    .await?;

    let labels = sqlx::query_as::<_, LabelInfo>(
        r#"SELECT il.id, il.name, il.color
           FROM issue_labels il
           JOIN issue_label_assignments ila ON il.id = ila.label_id
           WHERE ila.issue_id = $1"#,
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await?;

    let milestone_title = if let Some(mid) = issue.milestone_id {
        sqlx::query_scalar::<_, String>("SELECT title FROM milestones WHERE id = $1")
            .bind(mid)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    let description_html = issue.description.as_ref().map(|d| {
        crate::markdown::render::render_markdown(d)
    });

    Ok(IssueDetail {
        id: issue.id,
        number: issue.number,
        title: issue.title,
        description: issue.description,
        description_html,
        state: issue.state,
        author_id: issue.author_id,
        author_username: author.username,
        author_display_name: author.display_name,
        created_at: issue.created_at,
        updated_at: issue.updated_at,
        closed_at: issue.closed_at,
        labels,
        milestone_title,
    })
}

pub async fn close_issue(pool: &PgPool, repo_id: Uuid, number: i32) -> AppResult<()> {
    let r = sqlx::query(
        "UPDATE issues SET state = 'closed', closed_at = now(), updated_at = now()
         WHERE repository_id = $1 AND number = $2 AND state = 'open'",
    )
    .bind(repo_id)
    .bind(number)
    .execute(pool)
    .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound("Issue not found or already closed.".into()));
    }
    Ok(())
}

pub async fn reopen_issue(pool: &PgPool, repo_id: Uuid, number: i32) -> AppResult<()> {
    sqlx::query(
        "UPDATE issues SET state = 'open', closed_at = NULL, updated_at = now()
         WHERE repository_id = $1 AND number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn assign_label(pool: &PgPool, issue_id: Uuid, label_id: Uuid) -> AppResult<()> {
    sqlx::query("INSERT INTO issue_label_assignments (issue_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(issue_id).bind(label_id).execute(pool).await?;
    Ok(())
}
