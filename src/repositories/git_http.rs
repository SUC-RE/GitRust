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

#[derive(Deserialize)]
pub struct GitParams { pub owner: String, pub repo: String }

#[derive(Deserialize)]
pub struct ServiceQuery { pub service: String }

pub async fn info_refs(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(params): Path<GitParams>,
    Query(query): Query<ServiceQuery>,
) -> AppResult<Response> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    if repository.is_private && current_user.is_none() {
        return Err(crate::error::AppError::Unauthorized);
    }

    let repo_path = crate::git_core::repo::repo_path(
        &state.config.data_dir,
        &repository.owner_id.to_string(),
        &repository.name,
    );

    let git_cmd = match query.service.as_str() {
        "git-upload-pack" => "git-upload-pack",
        "git-receive-pack" => "git-receive-pack",
        _ => return Err(crate::error::AppError::BadRequest("Invalid service.".into())),
    };

    let output = std::process::Command::new("git")
        .arg(git_cmd.replace("git-", ""))
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(&repo_path)
        .output()
        .unwrap_or_else(|_| std::process::Output { status: std::process::ExitStatus::default(), stdout: vec![], stderr: vec![] });

    let content_type = format!("application/x-{}-advertisement", query.service);
    let pkt = format!("# service={}\n", query.service);
    let pkt_len = format!("{:04x}", pkt.len() + 4);
    let body_data = format!("{}{}{}", pkt_len, pkt, String::from_utf8_lossy(&output.stdout));

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", content_type.parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());

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
    Path(params): Path<GitParams>,
    body: String,
) -> AppResult<Response> {
    let current_user = current_user_from_session(&session).await;
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    if repository.is_private && current_user.is_none() {
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
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("Failed: {}", e)))?;

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(body.as_bytes()).ok();
    }

    let output = child.wait_with_output()
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("Failed: {}", e)))?;

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
    Path(params): Path<GitParams>,
    body: String,
) -> AppResult<Response> {
    let _current_user = current_user_from_session(&session).await
        .ok_or(crate::error::AppError::Unauthorized)?;
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
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("Failed: {}", e)))?;

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(body.as_bytes()).ok();
    }

    let output = child.wait_with_output()
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("Failed: {}", e)))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/x-git-receive-pack-result")
        .header("Cache-Control", "no-cache")
        .body(Body::from(output.stdout))
        .unwrap())
}
