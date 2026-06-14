use axum::{
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use std::sync::Arc;
use tower_sessions::Session;

use crate::state::AppState;
use crate::users::model::UserInfo;

pub async fn require_auth(
    session: Session,
    State(_state): State<Arc<AppState>>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let user_id: Option<String> = session.get("user_id").await.unwrap_or(None);
    match user_id {
        Some(_) => {
            let response = next.run(request).await;
            Ok(response)
        }
        None => Ok(Redirect::to("/auth/login").into_response()),
    }
}

pub async fn current_user_from_session(session: &Session) -> Option<UserInfo> {
    let stored: Option<serde_json::Value> = session.get("user").await.ok().flatten();
    stored.and_then(|v| serde_json::from_value::<UserInfo>(v).ok())
}
