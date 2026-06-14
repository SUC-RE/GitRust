use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::repositories::model::{RepoInfo, Repository};

pub async fn create_repo(
    pool: &PgPool,
    data_dir: &str,
    owner_type: &str,
    owner_id: Uuid,
    name: &str,
    description: &str,
    is_private: bool,
) -> AppResult<Repository> {
    if name.len() < 1 || name.len() > 128 {
        return Err(AppError::BadRequest("Repository name must be 1-128 characters.".into()));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(AppError::BadRequest("Repository name contains invalid characters.".into()));
    }
    if Repository::find_by_owner_and_name(pool, owner_type, owner_id, name).await?.is_some() {
        return Err(AppError::Conflict("Repository already exists.".into()));
    }

    let repo = sqlx::query_as::<_, Repository>(
        r#"INSERT INTO repositories (owner_type, owner_id, name, description, is_private)
           VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
    )
    .bind(owner_type)
    .bind(owner_id)
    .bind(name)
    .bind(description)
    .bind(is_private)
    .fetch_one(pool)
    .await?;

    let repo_path = crate::git_core::repo::repo_path(data_dir, &owner_id.to_string(), name);
    crate::git_core::repo::init_bare(&repo_path)?;

    Ok(repo)
}

pub async fn resolve_repo(
    pool: &PgPool,
    owner_name: &str,
    repo_name: &str,
) -> AppResult<(Repository, String)> {
    // Try user owner
    let user_row = sqlx::query("SELECT id, username FROM users WHERE username = $1")
        .bind(owner_name)
        .fetch_optional(pool)
        .await?;
    if let Some(row) = user_row {
        let uid: Uuid = row.try_get("id")?;
        let uname: String = row.try_get("username")?;
        if let Some(repo) = Repository::find_by_owner_and_name(pool, "user", uid, repo_name).await? {
            return Ok((repo, uname));
        }
    }

    // Try group owner
    let group_row = sqlx::query("SELECT id, name FROM project_groups WHERE name = $1")
        .bind(owner_name)
        .fetch_optional(pool)
        .await?;
    if let Some(row) = group_row {
        let gid: Uuid = row.try_get("id")?;
        let gname: String = row.try_get("name")?;
        if let Some(repo) = Repository::find_by_owner_and_name(pool, "group", gid, repo_name).await? {
            return Ok((repo, gname));
        }
    }

    Err(AppError::NotFound("Repository not found.".into()))
}

pub fn check_read_permission(repo: &Repository, _user_id: Option<Uuid>) -> bool {
    if !repo.is_private { return true; }
    true
}

pub async fn get_repo_info(pool: &PgPool, repo: &Repository) -> AppResult<RepoInfo> {
    let owner_name = match repo.owner_type.as_str() {
        "user" => {
            let row = sqlx::query("SELECT username FROM users WHERE id = $1")
                .bind(repo.owner_id)
                .fetch_optional(pool)
                .await?;
            row.and_then(|r| r.try_get::<String, _>("username").ok())
                .unwrap_or_default()
        }
        "group" => {
            let row = sqlx::query("SELECT name FROM project_groups WHERE id = $1")
                .bind(repo.owner_id)
                .fetch_optional(pool)
                .await?;
            row.and_then(|r| r.try_get::<String, _>("name").ok())
                .unwrap_or_default()
        }
        _ => String::new(),
    };
    Ok(RepoInfo {
        id: repo.id, name: repo.name.clone(),
        description: repo.description.clone(),
        owner_name, owner_type: repo.owner_type.clone(),
        default_branch: repo.default_branch.clone(),
        is_private: repo.is_private, is_archived: repo.is_archived,
        created_at: repo.created_at,
    })
}
