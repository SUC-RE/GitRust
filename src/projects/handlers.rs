use axum::{extract::State, response::Html};
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
        current_user,
        repos,
        groups,
    }).await?;
    Ok(Html(html))
}
