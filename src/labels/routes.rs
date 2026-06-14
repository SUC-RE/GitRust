use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::state::AppState;
use super::handlers;

pub fn label_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}/-/labels", get(handlers::list))
        .route("/{owner}/{repo}/-/labels", post(handlers::create))
        .route("/{owner}/{repo}/-/labels/{label_id}", axum::routing::delete(handlers::delete))
}
