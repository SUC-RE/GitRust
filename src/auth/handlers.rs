use axum::{
    extract::{Path, State},
    response::{Html, Redirect, IntoResponse},
    Form,
};
use minijinja::context;
use std::sync::Arc;
use tower_sessions::Session;
use uuid::Uuid;

use crate::auth::dto::{LoginForm, RegisterForm};
use crate::auth::service::AuthService;
use crate::captcha::service::CaptchaService;
use crate::email::service::EmailService;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::users::model::UserInfo;

pub async fn login_form(
    State(state): State<Arc<AppState>>,
) -> AppResult<Html<String>> {
    let html = state.templates.render("pages/auth/login.jinja", context! {}).await?;
    Ok(Html(html))
}

pub async fn login_submit(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> AppResult<impl IntoResponse> {
    let user = AuthService::login(&state.pool, &form.username, &form.password).await?;

    let user_info: UserInfo = user.into();
    let user_json = serde_json::to_value(&user_info)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Serialize error: {}", e)))?;

    session.insert("user_id", user_info.id.to_string()).await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Session error: {}", e)))?;
    session.insert("user", &user_json).await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Session error: {}", e)))?;

    Ok(Redirect::to("/"))
}

pub async fn register_form(
    State(state): State<Arc<AppState>>,
) -> AppResult<Html<String>> {
    let (token, answer, base64) = CaptchaService::generate();
    AuthService::store_captcha(&state.pool, &token, &answer).await?;

    let html = state.templates.render("pages/auth/register.jinja", context! {
        captcha_token => token,
        captcha_image => base64,
    }).await?;
    Ok(Html(html))
}

pub async fn register_submit(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(form): Form<RegisterForm>,
) -> AppResult<impl IntoResponse> {
    if form.password != form.password_confirm {
        let (token, answer, base64) = CaptchaService::generate();
        AuthService::store_captcha(&state.pool, &token, &answer).await?;
        let html = state.templates.render("pages/auth/register.jinja", context! {
            error => "Passwords do not match.",
            captcha_token => token,
            captcha_image => base64,
        }).await?;
        return Ok(Html(html).into_response());
    }

    let valid = AuthService::verify_captcha(
        &state.pool,
        &form.captcha_token,
        &form.captcha_input,
    )
    .await?;

    if !valid {
        let (token, answer, base64) = CaptchaService::generate();
        AuthService::store_captcha(&state.pool, &token, &answer).await?;
        let html = state.templates.render("pages/auth/register.jinja", context! {
            error => "Verification code is incorrect.",
            captcha_token => token,
            captcha_image => base64,
        }).await?;
        return Ok(Html(html).into_response());
    }

    let user = AuthService::register(
        &state.pool,
        &form.username,
        &form.email,
        &form.password,
        &form.display_name,
    )
    .await?;

    let verify_token = Uuid::new_v4().to_string();
    AuthService::create_email_verification(&state.pool, user.id, &verify_token).await?;

    let email_service = EmailService::new(
        state.config.smtp_host.clone(),
        state.config.smtp_port,
        state.config.smtp_user.clone(),
        state.config.smtp_pass.clone(),
        &state.config.smtp_from,
        state.config.base_url.clone(),
    );
    let _ = email_service
        .send_verification_email(&user.email, &user.username, &verify_token)
        .await;

    let user_info: UserInfo = user.into();
    let user_json = serde_json::to_value(&user_info)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Serialize error: {}", e)))?;

    session.insert("user_id", user_info.id.to_string()).await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Session error: {}", e)))?;
    session.insert("user", &user_json).await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Session error: {}", e)))?;

    Ok(Redirect::to("/").into_response())
}

pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> AppResult<impl IntoResponse> {
    AuthService::verify_email(&state.pool, &token).await?;
    let html = state.templates.render("pages/auth/login.jinja", context! {
        success => "Email verified successfully! You can now sign in.",
    }).await?;
    Ok(Html(html))
}

pub async fn logout(
    session: Session,
) -> Result<Redirect, AppError> {
    session.clear().await;
    Ok(Redirect::to("/"))
}

pub async fn captcha_refresh(
    State(state): State<Arc<AppState>>,
) -> AppResult<impl IntoResponse> {
    let (token, answer, base64) = CaptchaService::generate();
    AuthService::store_captcha(&state.pool, &token, &answer).await?;

    Ok(axum::Json(serde_json::json!({
        "token": token,
        "image": base64,
    })))
}
