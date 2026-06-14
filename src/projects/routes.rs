use axum::{routing::get, Router};
use std::sync::Arc;
use crate::state::AppState;
use super::handlers;
pub fn project_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/projects", get(handlers::dashboard))
}
