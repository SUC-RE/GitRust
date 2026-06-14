use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::state::AppState;
use super::handlers;

pub fn mr_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}/-/merge_requests", get(handlers::list))
        .route("/{owner}/{repo}/-/merge_requests/new", get(handlers::new_form))
        .route("/{owner}/{repo}/-/merge_requests/new", post(handlers::create))
        .route("/{owner}/{repo}/-/merge_requests/{number}", get(handlers::detail))
        .route("/{owner}/{repo}/-/merge_requests/{number}/close", post(handlers::close))
        .route("/{owner}/{repo}/-/merge_requests/{number}/merge", post(handlers::merge))
}
