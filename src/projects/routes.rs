use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::state::AppState;
use super::handlers;
pub fn project_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/projects", get(handlers::dashboard))
        .route("/projects/new", get(handlers::new_repo_form))
        .route("/projects/new", post(handlers::create_repo))
}
