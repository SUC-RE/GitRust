use axum::{
    extract::{Path, State}, response::{Html, Redirect}, Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;


use crate::error::AppResult;
use crate::milestones::service;
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service as repo_svc;
use crate::state::AppState;

#[derive(Deserialize)] pub struct MRepoParams { pub owner: String, pub repo: String }
#[derive(Deserialize)] pub struct CreateMilestoneForm { pub title: String, pub description: Option<String>, pub due_date: Option<String> }

pub async fn list(
    State(state): State<Arc<AppState>>, session: Session, Path(params): Path<MRepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let milestones = service::list(&state.pool, repo.id).await?;
    let html = state.templates.render("pages/repo/milestones/list.jinja", context! {
        current_user, repo => repo_info, milestones, sidebar_active => "milestones",
    }).await?;
    Ok(Html(html))
}

pub async fn create(
    State(state): State<Arc<AppState>>, session: Session, Path(params): Path<MRepoParams>,
    Form(form): Form<CreateMilestoneForm>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let due = form.due_date.and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());
    service::create(&state.pool, repo.id, &form.title, form.description.as_deref(), due).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/milestones", params.owner, params.repo)))
}
