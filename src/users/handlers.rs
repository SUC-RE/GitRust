use axum::{
    extract::{Path, State},
    response::{Html, Redirect, IntoResponse},
    Form,
};
use minijinja::context;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::middleware::auth::current_user_from_session;
use crate::state::AppState;
use crate::users::service;

pub async fn profile(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(username): Path<String>,
) -> AppResult<Html<String>> {
    let profile = service::get_user_profile(&state.pool, &username).await?;
    let repos_count = service::get_user_repos_count(&state.pool, profile.id).await.unwrap_or(0);
    let groups_count = service::get_user_groups_count(&state.pool, profile.id).await.unwrap_or(0);
    let current_user = current_user_from_session(&session).await;
    let is_owner = current_user.as_ref().map(|u| u.id == profile.id).unwrap_or(false);

    let html = state.templates.render("pages/user/profile.jinja", context! {
        current_user,
        profile,
        repos_count,
        groups_count,
        is_owner,
    }).await?;
    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct ProfileForm {
    display_name: String,
    bio: String,
}

pub async fn settings_profile(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;

    let html = state.templates.render("pages/user/settings.jinja", context! {
        current_user,
        tab => "profile",
    }).await?;
    Ok(Html(html))
}

pub async fn settings_profile_save(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(form): Form<ProfileForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    service::update_profile(&state.pool, current_user.id, &form.display_name, &form.bio).await?;
    Ok(Redirect::to(&format!("/{}", current_user.username)))
}

#[derive(Deserialize)]
pub struct PasswordForm {
    current_password: String,
    new_password: String,
    new_password_confirm: String,
}

pub async fn settings_password(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;

    let html = state.templates.render("pages/user/settings.jinja", context! {
        current_user,
        tab => "password",
    }).await?;
    Ok(Html(html))
}

pub async fn settings_password_save(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(form): Form<PasswordForm>,
) -> AppResult<impl IntoResponse> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;

    if form.new_password != form.new_password_confirm {
        let html = state.templates.render("pages/user/settings.jinja", context! {
            current_user,
            tab => "password",
            error => "New passwords do not match.",
        }).await?;
        return Ok(Html(html).into_response());
    }

    service::change_password(&state.pool, current_user.id, &form.current_password, &form.new_password).await?;

    let html = state.templates.render("pages/user/settings.jinja", context! {
        current_user,
        tab => "password",
        success => "Password changed successfully.",
    }).await?;
    Ok(Html(html).into_response())
}

pub async fn settings_ssh_keys(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let ssh_keys = service::list_ssh_keys(&state.pool, current_user.id).await.unwrap_or_default();

    let html = state.templates.render("pages/user/settings.jinja", context! {
        current_user,
        tab => "ssh_keys",
        ssh_keys,
    }).await?;
    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct SshKeyForm {
    title: String,
    public_key: String,
}

pub async fn settings_ssh_keys_add(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(form): Form<SshKeyForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let _ = service::add_ssh_key(&state.pool, current_user.id, &form.title, &form.public_key).await?;
    Ok(Redirect::to("/settings/ssh-keys"))
}

pub async fn settings_ssh_keys_delete(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(key_id): Path<Uuid>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    service::delete_ssh_key(&state.pool, current_user.id, key_id).await?;
    Ok(Redirect::to("/settings/ssh-keys"))
}
