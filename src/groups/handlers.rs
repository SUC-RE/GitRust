use axum::{
    extract::{Path, State},
    response::{Html, Redirect, IntoResponse},
    Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::error::{AppError, AppResult};
use crate::groups::{model::Group, service};
use crate::middleware::auth::current_user_from_session;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateGroupForm {
    name: String,
    display_name: String,
    description: String,
}

pub async fn new_form(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;

    let html = state.templates.render("pages/projects/new_group.jinja", context! {
        current_user,
    }).await?;
    Ok(Html(html))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(form): Form<CreateGroupForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;

    let group = service::create_group(
        &state.pool,
        &form.name,
        &form.display_name,
        &form.description,
        current_user.id,
    )
    .await?;

    Ok(Redirect::to(&format!("/groups/{}", group.name)))
}

pub async fn show(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(name): Path<String>,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;
    let (group, members) = service::get_group_with_members(&state.pool, &name).await?;
    let invite_codes = service::get_group_invite_codes(&state.pool, group.id).await.unwrap_or_default();

    let is_member = current_user.as_ref().map_or(false, |u| {
        members.iter().any(|m| m.user_id == u.id)
    });

    let is_owner = members.iter().any(|m| m.role == "owner" && current_user.as_ref().map_or(false, |u| m.user_id == u.id));

    let html = state.templates.render("pages/projects/group.jinja", context! {
        current_user,
        group,
        members,
        invite_codes,
        is_member,
        is_owner,
    }).await?;
    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct JoinForm {
    code: String,
}

pub async fn join_form(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await;

    let html = state.templates.render("pages/projects/join.jinja", context! {
        current_user,
    }).await?;
    Ok(Html(html))
}

pub async fn join_submit(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(form): Form<JoinForm>,
) -> AppResult<impl IntoResponse> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;

    match service::join_by_code(&state.pool, &form.code, current_user.id).await {
        Ok(group) => Ok(Redirect::to(&format!("/groups/{}", group.name)).into_response()),
        Err(e) => {
            let html = state.templates.render("pages/projects/join.jinja", context! {
                current_user,
                error => e.message(),
            }).await?;
            Ok(Html(html).into_response())
        }
    }
}

#[derive(Deserialize)]
pub struct GenerateInviteForm {
    max_uses: Option<i32>,
    expires_days: Option<i32>,
}

pub async fn generate_invite(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(name): Path<String>,
    Form(form): Form<GenerateInviteForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;

    let group = Group::find_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| AppError::NotFound("Group not found.".into()))?;

    let _ = service::generate_invite_code(
        &state.pool,
        group.id,
        current_user.id,
        form.max_uses,
        form.expires_days,
    )
    .await?;

    Ok(Redirect::to(&format!("/groups/{}", name)))
}
