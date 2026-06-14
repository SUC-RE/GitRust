use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::state::AppState;
use super::handlers;

pub fn group_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/groups/new", get(handlers::new_form))
        .route("/groups/new", post(handlers::create))
        .route("/groups/join", get(handlers::join_form))
        .route("/groups/join", post(handlers::join_submit))
        .route("/groups/{name}", get(handlers::show))
        .route("/groups/{name}/invite", post(handlers::generate_invite))
}
