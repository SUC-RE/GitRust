use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::state::AppState;
use super::handlers;
pub fn milestone_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}/-/milestones", get(handlers::list))
        .route("/{owner}/{repo}/-/milestones/new", post(handlers::create))
}
