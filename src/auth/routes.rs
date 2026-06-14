use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::state::AppState;
use super::handlers;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", get(handlers::login_form).post(handlers::login_submit))
        .route("/auth/register", get(handlers::register_form).post(handlers::register_submit))
        .route("/auth/captcha/refresh", post(handlers::captcha_refresh))
        .route("/auth/verify-email/{token}", get(handlers::verify_email))
        .route("/auth/logout", post(handlers::logout))
}
