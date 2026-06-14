use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::state::AppState;
use super::handlers;
pub fn member_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{owner}/{repo}/-/members", get(handlers::list))
        .route("/{owner}/{repo}/-/members/add", post(handlers::add))
        .route("/{owner}/{repo}/-/members/{user_id}/remove", post(handlers::remove))
}
