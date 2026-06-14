use axum::{Router, routing::get};
use gitrust::activity::handlers::dashboard_feed;
use gitrust::auth::routes::auth_routes;
use gitrust::config::Config;
use gitrust::state::AppState;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, services::ServeDir};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::EnvFilter;

use gitrust::groups::routes::group_routes;
use gitrust::issues::routes::issue_routes;
use gitrust::merge_requests::routes::mr_routes;
use gitrust::milestones::routes::milestone_routes;
use gitrust::wiki::routes::wiki_routes;
use gitrust::members::routes::member_routes;
use gitrust::labels::routes::label_routes;
use gitrust::projects::routes::project_routes;
use gitrust::repositories::routes::repo_routes;
use gitrust::users::routes::{profile_routes, user_routes};

async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    tracing::info!("Connecting to database...");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;
    tracing::info!("Database connected.");

    let session_store = MemoryStore::default();

    let state = Arc::new(AppState::new(pool, config.clone()).await?);

    let app = Router::new()
        .route("/", get(dashboard_feed))
        .route("/health", get(health))
        .merge(auth_routes())
        .merge(user_routes())
        .merge(group_routes())
        .merge(project_routes())
        .merge(repo_routes())
        .merge(issue_routes())
        .merge(mr_routes())
        .merge(milestone_routes())
        .merge(wiki_routes())
        .merge(member_routes())
        .merge(label_routes())
        .merge(profile_routes())
        .nest_service("/static", ServeDir::new("static"))
        .layer(SessionManagerLayer::new(session_store))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(gitrust::middleware::logging::trace_layer())
        .with_state(state);

    let addr = config.addr();
    tracing::info!("GitRust listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
