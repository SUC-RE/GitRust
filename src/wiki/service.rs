use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppResult;
use crate::wiki::model::{WikiPage, WikiPageWithAuthor};

pub async fn get_page(pool: &PgPool, repo_id: Uuid, slug: &str) -> AppResult<Option<WikiPageWithAuthor>> {
    Ok(sqlx::query_as::<_, WikiPageWithAuthor>(
        r#"SELECT wp.id, wp.title, wp.slug, wp.content, wp.revision,
                  u.username as author_username, u.display_name as author_display_name, wp.updated_at
           FROM wiki_pages wp JOIN users u ON wp.author_id = u.id
           WHERE wp.repository_id = $1 AND wp.slug = $2
           ORDER BY wp.revision DESC LIMIT 1"#,
    ).bind(repo_id).bind(slug).fetch_optional(pool).await?)
}

pub async fn save_page(pool: &PgPool, repo_id: Uuid, author_id: Uuid, title: &str, slug: &str, content: &str) -> AppResult<WikiPage> {
    let next_rev: (i32,) = sqlx::query_as(
        "SELECT COALESCE(MAX(revision), 0) + 1 FROM wiki_pages WHERE repository_id = $1 AND slug = $2"
    ).bind(repo_id).bind(slug).fetch_one(pool).await?;

    Ok(sqlx::query_as::<_, WikiPage>(
        "INSERT INTO wiki_pages (repository_id, title, slug, content, author_id, revision) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
    ).bind(repo_id).bind(title).bind(slug).bind(content).bind(author_id).bind(next_rev.0).fetch_one(pool).await?)
}

pub async fn get_revisions(pool: &PgPool, repo_id: Uuid, slug: &str) -> AppResult<Vec<WikiPageWithAuthor>> {
    Ok(sqlx::query_as::<_, WikiPageWithAuthor>(
        r#"SELECT wp.id, wp.title, wp.slug, wp.content, wp.revision,
                  u.username as author_username, u.display_name as author_display_name, wp.updated_at
           FROM wiki_pages wp JOIN users u ON wp.author_id = u.id
           WHERE wp.repository_id = $1 AND wp.slug = $2 ORDER BY wp.revision DESC"#,
    ).bind(repo_id).bind(slug).fetch_all(pool).await?)
}

pub async fn list_pages(pool: &PgPool, repo_id: Uuid) -> AppResult<Vec<WikiPageWithAuthor>> {
    Ok(sqlx::query_as::<_, WikiPageWithAuthor>(
        r#"SELECT DISTINCT ON (wp.slug) wp.id, wp.title, wp.slug, wp.content, wp.revision,
                  u.username as author_username, u.display_name as author_display_name, wp.updated_at
           FROM wiki_pages wp JOIN users u ON wp.author_id = u.id
           WHERE wp.repository_id = $1 ORDER BY wp.slug, wp.revision DESC"#,
    ).bind(repo_id).fetch_all(pool).await?)
}
