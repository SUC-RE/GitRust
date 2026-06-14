use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::error::AppResult;
use crate::middleware::auth::current_user_from_session;
use crate::repositories::service;
use crate::state::AppState;
use crate::users::model::User;

#[derive(Deserialize)]
pub struct GitParams { pub owner: String, pub repo: String }

#[derive(Deserialize)]
pub struct ServiceQuery { pub service: String }

async fn check_git_auth(
    state: &AppState,
    session: &Session,
    headers: &HeaderMap,
) -> bool {
    if let Some(user) = current_user_from_session(session).await {
        tracing::debug!("Git auth: session user={}", user.username);
        return true;
    }
    if let Some(auth) = headers.get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            tracing::debug!("Git auth: Authorization header present");
            if auth_str.starts_with("Basic ") {
                let encoded = auth_str.trim_start_matches("Basic ").trim();
                match base64_decode(encoded) {
                    Ok(decoded) => {
                        tracing::debug!("Git auth: decoded Basic creds");
                        if let Some((username, password)) = decoded.split_once(':') {
                            tracing::debug!("Git auth: username={}", username);
                            match User::find_by_username(&state.pool, username).await {
                                Ok(Some(user)) => {
                                    use argon2::{
                                        password_hash::{PasswordHash, PasswordVerifier},
                                        Argon2,
                                    };
                                    match PasswordHash::new(&user.password_hash) {
                                        Ok(parsed) => {
                                            let ok = Argon2::default()
                                                .verify_password(password.as_bytes(), &parsed)
                                                .is_ok();
                                            tracing::debug!("Git auth: password match={}", ok);
                                            return ok;
                                        }
                                        Err(e) => tracing::debug!("Git auth: hash parse error={}", e),
                                    }
                                }
                                Ok(None) => tracing::debug!("Git auth: user not found"),
                                Err(e) => tracing::debug!("Git auth: db error={}", e),
                            }
                        }
                    }
                    Err(_) => tracing::debug!("Git auth: base64 decode failed"),
                }
            }
        }
    } else {
        tracing::debug!("Git auth: no Authorization header");
    }
    false
}

fn base64_decode(input: &str) -> Result<String, ()> {
    use base64::{Engine as _, engine::general_purpose};
    let bytes = general_purpose::STANDARD.decode(input).map_err(|_| ())?;
    String::from_utf8(bytes).map_err(|_| ())
}

pub async fn info_refs(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    Path(params): Path<GitParams>,
    Query(query): Query<ServiceQuery>,
) -> AppResult<Response> {
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    if repository.is_private && !check_git_auth(&state, &session, &headers).await {
        return Err(crate::error::AppError::Unauthorized);
    }

    let repo_path = crate::git_core::repo::repo_path(
        &state.config.data_dir,
        &repository.owner_id.to_string(),
        &repository.name,
    );

    let git_subcmd = match query.service.as_str() {
        "git-upload-pack" => "upload-pack",
        "git-receive-pack" => "receive-pack",
        _ => return Err(crate::error::AppError::BadRequest("Invalid service.".into())),
    };

    let output = std::process::Command::new("git")
        .arg(git_subcmd)
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(&repo_path)
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: vec![],
        });

    let pkt = format!("# service={}\n", query.service);
    let pkt_len = format!("{:04x}", pkt.len() + 4);
    let body_data = format!("{}{}0000{}", pkt_len, pkt, String::from_utf8_lossy(&output.stdout));
    let content_type = format!("application/x-{}-advertisement", query.service);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .header("Cache-Control", "no-cache")
        .body(Body::from(body_data))
        .unwrap())
}

pub async fn upload_pack(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    Path(params): Path<GitParams>,
    body: axum::body::Bytes,
) -> AppResult<Response> {
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    if repository.is_private && !check_git_auth(&state, &session, &headers).await {
        return Err(crate::error::AppError::Unauthorized);
    }

    let repo_path = crate::git_core::repo::repo_path(
        &state.config.data_dir,
        &repository.owner_id.to_string(),
        &repository.name,
    );

    let mut child = std::process::Command::new("git")
        .arg("upload-pack")
        .arg("--stateless-rpc")
        .arg(&repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("git upload-pack failed: {}", e)))?;

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&body).ok();
    }

    let output = child.wait_with_output()
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("git upload-pack failed: {}", e)))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/x-git-upload-pack-result")
        .header("Cache-Control", "no-cache")
        .body(Body::from(output.stdout))
        .unwrap())
}

pub async fn receive_pack(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    Path(params): Path<GitParams>,
    body: axum::body::Bytes,
) -> AppResult<Response> {
    if !check_git_auth(&state, &session, &headers).await {
        return Err(crate::error::AppError::Unauthorized);
    }
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;

    let repo_path = crate::git_core::repo::repo_path(
        &state.config.data_dir,
        &repository.owner_id.to_string(),
        &repository.name,
    );

    let mut child = std::process::Command::new("git")
        .arg("receive-pack")
        .arg("--stateless-rpc")
        .arg(&repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("git receive-pack failed: {}", e)))?;

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&body).ok();
    }

    let output = child.wait_with_output()
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("git receive-pack failed: {}", e)))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/x-git-receive-pack-result")
        .header("Cache-Control", "no-cache")
        .body(Body::from(output.stdout))
        .unwrap())
}
