use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::state::AppState;
use super::handlers;

pub fn issue_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}/-/issues", get(handlers::list))
        .route("/{owner}/{repo}/-/issues/new", get(handlers::new_form))
        .route("/{owner}/{repo}/-/issues/new", post(handlers::create))
        .route("/{owner}/{repo}/-/issues/{number}", get(handlers::detail))
        .route("/{owner}/{repo}/-/issues/{number}/close", post(handlers::close))
        .route("/{owner}/{repo}/-/issues/{number}/reopen", post(handlers::reopen))
}
