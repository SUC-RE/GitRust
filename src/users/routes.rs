use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::state::AppState;
use super::handlers;

pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/settings/profile", get(handlers::settings_profile))
        .route("/settings/profile", post(handlers::settings_profile_save))
        .route("/settings/password", get(handlers::settings_password))
        .route("/settings/password", post(handlers::settings_password_save))
        .route("/settings/ssh-keys", get(handlers::settings_ssh_keys))
        .route("/settings/ssh-keys", post(handlers::settings_ssh_keys_add))
        .route("/settings/ssh-keys/{key_id}", axum::routing::delete(handlers::settings_ssh_keys_delete))
}

pub fn profile_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{username}", get(handlers::profile))
}
