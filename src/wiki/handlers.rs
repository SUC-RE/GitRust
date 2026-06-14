use axum::{
    extract::{Path, State}, response::{Html, Redirect}, Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::error::{AppError, AppResult};
use crate::helpers::slug::slugify;
use crate::markdown::render::render_markdown;
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service as repo_svc;
use crate::state::AppState;
use crate::wiki::service;

#[derive(Deserialize)] pub struct WRepoParams { pub owner: String, pub repo: String }
#[derive(Deserialize)] pub struct WikiForm { pub title: String, pub content: String }

pub async fn home(
    State(state): State<Arc<AppState>>, session: Session, Path(params): Path<WRepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let pages = service::list_pages(&state.pool, repo.id).await?;
    let home_page = service::get_page(&state.pool, repo.id, "home").await?;
    let html = state.templates.render("pages/repo/wiki/home.jinja", context! {
        current_user, repo => repo_info, pages, home_page, sidebar_active => "wiki",
    }).await?;
    Ok(Html(html))
}

pub async fn show(
    State(state): State<Arc<AppState>>, session: Session,
    Path((params, slug)): Path<(WRepoParams, String)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let page = service::get_page(&state.pool, repo.id, &slug).await?;
    let content_html = page.as_ref().map(|p| render_markdown(&p.content));
    let html = state.templates.render("pages/repo/wiki/page.jinja", context! {
        current_user, repo => repo_info, page, content_html, slug, sidebar_active => "wiki",
    }).await?;
    Ok(Html(html))
}

pub async fn edit_form(
    State(state): State<Arc<AppState>>, session: Session,
    Path((params, slug)): Path<(WRepoParams, String)>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = repo_svc::get_repo_info(&state.pool, &repo).await?;
    let page = service::get_page(&state.pool, repo.id, &slug).await?;
    let html = state.templates.render("pages/repo/wiki/edit.jinja", context! {
        current_user, repo => repo_info, page, slug, sidebar_active => "wiki",
    }).await?;
    Ok(Html(html))
}

pub async fn save(
    State(state): State<Arc<AppState>>, session: Session,
    Path((params, _slug)): Path<(WRepoParams, String)>, Form(form): Form<WikiForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let (repo, _) = repo_svc::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let new_slug = slugify(&form.title);
    service::save_page(&state.pool, repo.id, current_user.id, &form.title, &new_slug, &form.content).await?;
    Ok(Redirect::to(&format!("/{}/{}/-/wiki/{}", params.owner, params.repo, new_slug)))
}
