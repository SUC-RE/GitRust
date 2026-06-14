use axum::{
    extract::{Path, State}, response::{Html, Redirect}, Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use uuid::Uuid;

use crate::error::AppResult;
use crate::members::service;
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service as repo_svc;
use crate::state::AppState;
use sqlx::Row;

#[derive(Deserialize)] pub struct MemRepoParams { pub owner: String, pub repo: String }
#[derive(Deserialize)] pub struct AddMemberForm { pub username: String, pub permission: String }

pub async fn list(
    State(state): State<Arc<AppState>>, session: Session, Path(params): Path<MemRepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let members = service::list(&state.pool, repo.id).await?;
    let html = state.templates.render("pages/repo/members.jinja", context! {
        current_user, repo => repo_info, members, sidebar_active => "members",
    }).await?;
    Ok(Html(html))
}

pub async fn add(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<MemRepoParams>, Form(form): Form<AddMemberForm>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let user_row = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(&form.username).fetch_optional(&state.pool).await?;
    if let Some(row) = user_row {
        let uid: Uuid = row.try_get("id").unwrap_or_default();
        service::add(&state.pool, repo.id, uid, &form.permission).await?;
    }
    Ok(Redirect::to(&format!("/{}/{}/-/members", params.owner, params.repo)))
}

pub async fn remove(
    State(state): State<Arc<AppState>>, session: Session,
    Path((params, user_id)): Path<(MemRepoParams, Uuid)>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    service::remove(&state.pool, repo.id, user_id).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/members", params.owner, params.repo)))
}
