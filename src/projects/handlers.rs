use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use minijinja::context;
use std::sync::Arc;
use tower_sessions::Session;

use crate::error::{AppError, AppResult};
use crate::groups::model::Group;
use crate::middleware::auth::current_user_from_session;
use crate::state::AppState;

pub async fn dashboard(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;

    #[derive(serde::Serialize, sqlx::FromRow)]
    struct RepoRow {
        id: uuid::Uuid,
        name: String,
        description: Option<String>,
        owner_type: String,
        owner_id: uuid::Uuid,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let repos: Vec<RepoRow> = sqlx::query_as::<_, RepoRow>(
        r#"SELECT id, name, description, owner_type, owner_id, created_at
           FROM repositories
           WHERE (owner_type = 'user' AND owner_id = $1)
              OR (owner_type = 'group' AND owner_id IN (
                  SELECT group_id FROM group_members WHERE user_id = $1
              ))
           ORDER BY created_at DESC"#,
    )
    .bind(current_user.id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let groups = Group::list_by_user(&state.pool, current_user.id).await.unwrap_or_default();

    let html = state.templates.render("pages/projects/list.jinja", context! {
        current_user, repos, groups,
    }).await?;
    Ok(Html(html))
}

#[derive(serde::Deserialize)]
pub struct NewRepoForm {
    pub name: String,
    pub description: String,
    pub owner_type: String,
    pub owner_name: String,
    pub is_private: Option<String>,
}

pub async fn new_repo_form(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> AppResult<Html<String>> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    let html = state.templates.render("pages/projects/new_repo.jinja", context! {
        current_user,
    }).await?;
    Ok(Html(html))
}

pub async fn create_repo(
    State(state): State<Arc<AppState>>,
    session: Session,
    Form(form): Form<NewRepoForm>,
) -> AppResult<Redirect> {
    let current_user = current_user_from_session(&session).await
        .ok_or(AppError::Unauthorized)?;
    use crate::repositories::service as repo_svc;
    use sqlx::Row;

    let (owner_type, owner_id) = match form.owner_type.as_str() {
        "group" => {
            let row = sqlx::query("SELECT id FROM project_groups WHERE name = $1")
                .bind(&form.owner_name).fetch_optional(&state.pool).await?;
            row.map(|r| ("group".to_string(), r.try_get::<uuid::Uuid, _>("id").unwrap_or_default()))
                .unwrap_or(("user".to_string(), current_user.id))
        }
        _ => ("user".to_string(), current_user.id),
    };

    let is_private = form.is_private.as_deref() == Some("on");
    let repo = repo_svc::create_repo(&state.pool, &state.config.data_dir, &owner_type, owner_id, &form.name, &form.description, is_private).await?;
    let owner_display = if owner_type == "user" { current_user.username.clone() } else { form.owner_name.clone() };
    Ok(Redirect::to(&format!("/{}/{}", owner_display, repo.name)))
}
