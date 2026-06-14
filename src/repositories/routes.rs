use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::state::AppState;
use super::git_http;
use super::handlers;

pub fn repo_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}", get(handlers::overview))
        .route("/{owner}/{repo}/tree", get(handlers::tree_view))
        .route("/{owner}/{repo}/commits", get(handlers::commits))
        .route("/{owner}/{repo}/commit/{sha}", get(handlers::commit_detail))
        .route("/{owner}/{repo}/branches", get(handlers::branches))
        // Git Smart HTTP
        .route("/{owner}/{repo}.git/info/refs", get(git_http::info_refs))
        .route("/{owner}/{repo}.git/git-upload-pack", post(git_http::upload_pack))
        .route("/{owner}/{repo}.git/git-receive-pack", post(git_http::receive_pack))
        // Repository settings
        .route("/{owner}/{repo}/-/settings", get(handlers::settings_page))
        .route("/{owner}/{repo}/-/settings", post(handlers::settings_save))
}

pub fn repo_settings_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}/-/settings", get(handlers::settings_page))
        .route("/{owner}/{repo}/-/settings", post(handlers::settings_save))
}
