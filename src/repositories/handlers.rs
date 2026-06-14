use axum::{
    extract::{Path, Query, State},
    response::{Html, Redirect},
    Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::error::{AppError, AppResult};
use crate::git_core::{commit, diff, repo as git_repo, tree};
use crate::markdown::render::render_markdown;
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RepoParams { pub owner: String, pub repo: String }

#[derive(Deserialize)]
pub struct TreeQuery { pub ref_name: Option<String> }

#[derive(Deserialize)]
pub struct CommitsQuery { pub ref_name: Option<String>, pub page: Option<u32> }

#[derive(Deserialize)]
pub struct CommitParams { pub owner: String, pub repo: String, pub sha: String }

pub async fn overview(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _owner_name) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let default_branch = git_repo::default_branch(&git_repo_obj);
    let entries = tree::list_tree(&git_repo_obj, &default_branch, "").unwrap_or_default();
    let branches = git_repo::branches(&git_repo_obj).unwrap_or_default();
    let readme_html = tree::find_readme(&git_repo_obj, &default_branch)
        .and_then(|(name, content)| {
            if name.to_lowercase().ends_with(".md") || name.to_lowercase().ends_with(".markdown") {
                let text = String::from_utf8_lossy(&content).to_string();
                Some(render_markdown(&text))
            } else {
                Some(format!("<pre>{}</pre>", String::from_utf8_lossy(&content)))
            }
        });
    let commit_count = commit::get_commit_count(&git_repo_obj, &default_branch).unwrap_or(0);
    let html = state.templates.render("pages/repo/overview.jinja", context! {
        current_user, repo => repo_info, default_branch, entries,
        branches, readme_html, commit_count, sidebar_active => "files",
    }).await?;
    Ok(Html(html))
}

pub async fn tree_view(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Query(query): Query<TreeQuery>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let default_branch = git_repo::default_branch(&git_repo_obj);
    let ref_name = query.ref_name.unwrap_or(default_branch);
    let entries = tree::list_tree(&git_repo_obj, &ref_name, "").unwrap_or_default();
    let branches = git_repo::branches(&git_repo_obj).unwrap_or_default();
    let html = state.templates.render("pages/repo/tree.jinja", context! {
        current_user, repo => repo_info, current_ref => ref_name,
        entries, branches, path => "", sidebar_active => "files",
    }).await?;
    Ok(Html(html))
}

pub async fn commits(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Query(query): Query<CommitsQuery>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let default_branch = git_repo::default_branch(&git_repo_obj);
    let ref_name = query.ref_name.unwrap_or(default_branch);
    let page = query.page.unwrap_or(1);
    let (commits_list, total) = commit::list_commits(&git_repo_obj, &ref_name, page, 20).unwrap_or_default();
    let pagination = crate::helpers::pagination::Pagination::new(page, 20, total);
    let html = state.templates.render("pages/repo/commits.jinja", context! {
        current_user, repo => repo_info, current_ref => ref_name,
        commits => commits_list, pagination, sidebar_active => "commits",
    }).await?;
    Ok(Html(html))
}

pub async fn commit_detail(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<CommitParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let detail = commit::get_commit_detail(&git_repo_obj, &params.sha)?;
    let diffs = diff::commit_diff(&git_repo_obj, &params.sha).unwrap_or_default();
    let html = state.templates.render("pages/repo/commit.jinja", context! {
        current_user, repo => repo_info, commit => detail,
        diff_files => diffs, sidebar_active => "commits",
    }).await?;
    Ok(Html(html))
}

pub async fn branches(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let repo_path = git_repo::repo_path(&state.config.data_dir, &repository.owner_id.to_string(), &repository.name);
    let git_repo_obj = git_repo::open_bare(&repo_path)?;
    let branch_list = git_repo::branches(&git_repo_obj).unwrap_or_default();
    let html = state.templates.render("pages/repo/branches.jinja", context! {
        current_user, repo => repo_info, branches_list => branch_list,
        sidebar_active => "files",
    }).await?;
    Ok(Html(html))
}


#[derive(Deserialize)]
pub struct SettingsForm {
    pub description: String,
    pub is_private: Option<String>,
}

pub async fn settings_page(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    let repo_info = service::get_repo_info(&state.pool, &repository).await?;
    let html = state.templates.render("pages/repo/settings.jinja", context! {
        current_user, repo => repo_info, sidebar_active => "settings",
    }).await?;
    Ok(Html(html))
}

pub async fn settings_save(
    State(state): State<Arc<AppState>>, session: Session,
    Path(params): Path<RepoParams>, Form(form): Form<SettingsForm>,
) -> AppResult<Redirect> {
    let _current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;

    let is_private = form.is_private.as_deref() == Some("on");
    sqlx::query("UPDATE repositories SET description = $1, is_private = $2, updated_at = now() WHERE id = $3")
        .bind(&form.description)
        .bind(is_private)
        .bind(repository.id)
        .execute(&state.pool)
        .await?;

    Ok(Redirect::to(&format!("/{}/{}", params.owner, params.repo)))
}
