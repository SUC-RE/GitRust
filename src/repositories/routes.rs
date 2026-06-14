use axum::{routing::get, Router};
use std::sync::Arc;

use crate::state::AppState;
use super::handlers;

pub fn repo_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}", get(handlers::overview))
        .route("/{owner}/{repo}/tree", get(handlers::tree_view))
        .route("/{owner}/{repo}/commits", get(handlers::commits))
        .route("/{owner}/{repo}/commit/{sha}", get(handlers::commit_detail))
        .route("/{owner}/{repo}/branches", get(handlers::branches))
}
