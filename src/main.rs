use anyhow::Context;
use techpulse::{
    config::{AppConfig, CategoryConfig},
    web::AppState,
};
use tracing::warn;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config =
        AppConfig::load("config/sources.toml").context("load feed source configuration")?;
    let category_config =
        CategoryConfig::load("config/categories.toml").context("load category configuration")?;
    let state = AppState::load(config, category_config).await;
    let address = std::env::var("TECHPULSE_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    if let Err(error) = techpulse::web::serve(state, &address).await {
        warn!(%error, "web server stopped");
        return Err(error).context("run web server");
    }
    Ok(())
}
