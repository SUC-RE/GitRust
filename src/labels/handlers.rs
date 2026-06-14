use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
    Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::labels::service;
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service as repo_svc;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LabelRepoParams { pub owner: String, pub repo: String }

#[derive(Deserialize)]
pub struct CreateLabelForm { pub name: String, pub color: String, pub description: Option<String> }

pub async fn list(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<LabelRepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let labels = service::list_labels(&state.pool, repo.id).await?;
    let html = state.templates.render("pages/repo/labels.jinja", context! {
        current_user, repo => repo_info, labels, sidebar_active => "labels",
    }).await?;
    Ok(Html(html))
}

pub async fn create(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<LabelRepoParams>, Form(form): Form<CreateLabelForm>,
) -> AppResult<Redirect> {
    let _current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    service::create_label(&state.pool, repo.id, &form.name, &form.color, form.description.as_deref()).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/labels", params.owner, params.repo)))
}

pub async fn delete(
    State(state): State<Arc<AppState>>, session: Session,
    Path((params, label_id)): Path<(LabelRepoParams, Uuid)>,
) -> AppResult<Redirect> {
    let _current_user = current_user_from_session(&session).await.ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    service::delete_label(&state.pool, repo.id, label_id).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/labels", params.owner, params.repo)))
}
