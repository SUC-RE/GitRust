use axum::{
    extract::{Path, Query, State},
    response::{Html, Redirect, IntoResponse},
    Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::error::{AppError, AppResult};
use crate::issues::{model::Label, service};
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service as repo_svc;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct IssueRepoParams { pub owner: String, pub repo: String }

#[derive(Deserialize)]
pub struct IssueListQuery { pub state: Option<String>, pub page: Option<u32> }

#[derive(Deserialize)]
pub struct IssueNumber { pub number: i32 }

#[derive(Deserialize)]
pub struct CreateIssueForm { pub title: String, pub description: String }

pub async fn list(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<IssueRepoParams>, Query(query): Query<IssueListQuery>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let page = query.page.unwrap_or(1);
    let state_filter = query.state.as_deref();
    let (issues, pagination) = service::list_issues(&state.pool, repo.id, state_filter, page, 20).await?;
    let html = state.templates.render("pages/repo/issues/list.jinja", context! {
        current_user, repo => repo_info, issues, pagination,
        current_state => state_filter.unwrap_or("open"),
        sidebar_active => "issues",
    }).await?;
    Ok(Html(html))
}

pub async fn new_form(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<IssueRepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let html = state.templates.render("pages/repo/issues/new.jinja", context! {
        current_user, repo => repo_info, sidebar_active => "issues",
    }).await?;
    Ok(Html(html))
}

pub async fn create(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<IssueRepoParams>, Form(form): Form<CreateIssueForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let issue = service::create_issue(&state.pool, repo.id, current_user.id, &form.title, &form.description).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/issues/{}", params.owner, params.repo, issue.number)))
}

pub async fn detail(
    State(state): State<Arc<AppState>>, session: Session,
    Path((params, num)): Path<(IssueRepoParams, IssueNumber)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let issue = service::get_issue(&state.pool, repo.id, num.number).await?;
    let labels: Vec<Label> = sqlx::query_as("SELECT * FROM issue_labels WHERE repository_id = $1 ORDER BY name")
        .bind(repo.id).fetch_all(&state.pool).await.unwrap_or_default();
    let html = state.templates.render("pages/repo/issues/detail.jinja", context! {
        current_user, repo => repo_info, issue, all_labels => labels,
        sidebar_active => "issues",
    }).await?;
    Ok(Html(html))
}

pub async fn close(
    State(state): State<Arc<AppState>>, session: Session,
    Path((params, num)): Path<(IssueRepoParams, IssueNumber)>,
) -> AppResult<impl IntoResponse> {
    let _ = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    service::close_issue(&state.pool, repo.id, num.number).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/issues/{}", params.owner, params.repo, num.number)))
}

pub async fn reopen(
    State(state): State<Arc<AppState>>, session: Session,
    Path((params, num)): Path<(IssueRepoParams, IssueNumber)>,
) -> AppResult<Redirect> {
    let _ = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    service::reopen_issue(&state.pool, repo.id, num.number).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/issues/{}", params.owner, params.repo, num.number)))
}
