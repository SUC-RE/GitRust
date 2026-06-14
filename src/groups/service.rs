use chrono::Utc;
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::groups::model::{Group, GroupMember};

pub async fn create_group(
    pool: &PgPool,
    name: &str,
    display_name: &str,
    description: &str,
    created_by: Uuid,
) -> AppResult<Group> {
    if name.len() < 3 || name.len() > 128 {
        return Err(AppError::BadRequest("Group name must be 3-128 characters.".into()));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::BadRequest("Group name can only contain letters, numbers, -, and _.".into()));
    }
    if Group::find_by_name(pool, name).await?.is_some() {
        return Err(AppError::Conflict("Group name already taken.".into()));
    }

    let group = sqlx::query_as::<_, Group>(
        r#"INSERT INTO project_groups (name, display_name, description, created_by)
           VALUES ($1, $2, $3, $4) RETURNING *"#,
    )
    .bind(name)
    .bind(display_name)
    .bind(description)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    sqlx::query("INSERT INTO group_members (group_id, user_id, role) VALUES ($1, $2, 'owner')")
        .bind(group.id)
        .bind(created_by)
        .execute(pool)
        .await?;

    Ok(group)
}

pub async fn generate_invite_code(
    pool: &PgPool,
    group_id: Uuid,
    created_by: Uuid,
    max_uses: Option<i32>,
    expires_days: Option<i32>,
) -> AppResult<String> {
    let code: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    let expires_at = expires_days.map(|d| Utc::now() + chrono::Duration::days(d as i64));

    sqlx::query(
        r#"INSERT INTO invite_codes (code, group_id, created_by, max_uses, expires_at)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(&code)
    .bind(group_id)
    .bind(created_by)
    .bind(max_uses)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(code)
}

pub async fn join_by_code(pool: &PgPool, code: &str, user_id: Uuid) -> AppResult<Group> {
    #[derive(sqlx::FromRow)]
    struct InviteRow {
        group_id: Uuid,
        max_uses: Option<i32>,
        current_uses: i32,
        expires_at: Option<chrono::DateTime<Utc>>,
        is_active: bool,
    }

    let invite = sqlx::query_as::<_, InviteRow>(
        "SELECT group_id, max_uses, current_uses, expires_at, is_active FROM invite_codes WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid invite code.".into()))?;

    if !invite.is_active {
        return Err(AppError::BadRequest("This invite code is no longer active.".into()));
    }
    if let Some(max) = invite.max_uses {
        if invite.current_uses >= max {
            return Err(AppError::BadRequest("This invite code has reached its maximum uses.".into()));
        }
    }
    if let Some(exp) = invite.expires_at {
        if exp < Utc::now() {
            return Err(AppError::BadRequest("This invite code has expired.".into()));
        }
    }

    let existing = sqlx::query_as::<_, GroupMember>(
        "SELECT * FROM group_members WHERE group_id = $1 AND user_id = $2",
    )
    .bind(invite.group_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("You are already a member of this group.".into()));
    }

    sqlx::query("INSERT INTO group_members (group_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(invite.group_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE invite_codes SET current_uses = current_uses + 1 WHERE code = $1")
        .bind(code)
        .execute(pool)
        .await?;

    let group = Group::find_by_id(pool, invite.group_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Group not found.".into()))?;

    Ok(group)
}

pub async fn get_group_with_members(
    pool: &PgPool,
    name: &str,
) -> AppResult<(Group, Vec<GroupMemberRow>)> {
    let group = Group::find_by_name(pool, name)
        .await?
        .ok_or_else(|| AppError::NotFound("Group not found.".into()))?;

    let members = sqlx::query_as::<_, GroupMemberRow>(
        r#"SELECT gm.user_id, gm.role, gm.joined_at, u.username, u.display_name
           FROM group_members gm
           JOIN users u ON gm.user_id = u.id
           WHERE gm.group_id = $1
           ORDER BY gm.joined_at"#,
    )
    .bind(group.id)
    .fetch_all(pool)
    .await?;

    Ok((group, members))
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct GroupMemberRow {
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: chrono::DateTime<Utc>,
    pub username: String,
    pub display_name: String,
}

pub async fn get_group_invite_codes(
    pool: &PgPool,
    group_id: Uuid,
) -> AppResult<Vec<InviteCodeRow>> {
    let codes = sqlx::query_as::<_, InviteCodeRow>(
        "SELECT code, max_uses, current_uses, expires_at, is_active, created_at FROM invite_codes WHERE group_id = $1 ORDER BY created_at DESC",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;
    Ok(codes)
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct InviteCodeRow {
    pub code: String,
    pub max_uses: Option<i32>,
    pub current_uses: i32,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<Utc>,
}
