use minijinja_autoreload::AutoReloader;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::AppError;

pub struct AppState {
    pub pool: PgPool,
    pub templates: TemplateEngine,
    pub config: Config,
}

pub struct TemplateEngine {
    reloader: Arc<Mutex<AutoReloader>>,
}

impl TemplateEngine {
    pub fn new(template_dir: &str) -> Result<Self, anyhow::Error> {
        let dir = template_dir.to_string();
        let reloader = AutoReloader::new(move |notifier| {
            let mut env = minijinja::Environment::new();
            env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html);
            env.set_loader(minijinja::path_loader(&dir));
            notifier.watch_path(&dir, true);
            Ok(env)
        });

        Ok(TemplateEngine {
            reloader: Arc::new(Mutex::new(reloader)),
        })
    }

    pub async fn render(
        &self,
        template: &str,
        ctx: minijinja::Value,
    ) -> Result<String, AppError> {
        let reloader = self.reloader.lock().await;
        let env = reloader
            .acquire_env()
            .map_err(|e| AppError::Template(minijinja::Error::new(
                minijinja::ErrorKind::TemplateNotFound,
                e.to_string(),
            )))?;
        let tmpl = env.get_template(template)?;
        Ok(tmpl.render(ctx)?)
    }
}

impl AppState {
    pub async fn new(pool: PgPool, config: Config) -> Result<Self, anyhow::Error> {
        let templates = TemplateEngine::new("templates")?;
        Ok(AppState {
            pool,
            templates,
            config,
        })
    }
}
