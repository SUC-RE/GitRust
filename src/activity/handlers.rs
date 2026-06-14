use axum::{
    extract::{Query, State},
    response::Html,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::activity::service;
use crate::error::AppResult;
use crate::middleware::auth::current_user_from_session;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct FeedQuery {
    page: Option<u32>,
}

pub async fn dashboard_feed(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(query): Query<FeedQuery>,
) -> AppResult<Html<String>> {
    let page = query.page.unwrap_or(1);
    let per_page = 20u32;

    let current_user = current_user_from_session(&session).await;

    let (events, pagination) = if let Some(ref user) = current_user {
        service::get_dashboard_feed(&state.pool, user.id, page, per_page)
            .await
            .unwrap_or_else(|_| (vec![], crate::helpers::pagination::Pagination::new(1, per_page, 0)))
    } else {
        (vec![], crate::helpers::pagination::Pagination::new(1, per_page, 0))
    };

    let html = state.templates.render("pages/home.jinja", context! {
        current_user,
        events,
        pagination,
    }).await?;
    Ok(Html(html))
}
