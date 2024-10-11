mod db;
mod calc;
mod wb;
mod state;
mod utils;
mod api;
mod update;

use std::sync::Arc;
use tracing::Level;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt().json()
        .with_max_level(Level::ERROR)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_url = utils::get_env_var("DATABASE_URL")?;
    let app_state = Arc::new(AppState::setup_app_state(&db_url)
        .await
        .expect("Failed to build AppState"));
    app_state.run_migrations().await?;

    let api_handle = tokio::spawn({
        let app_state = app_state.clone();
        async move {
            api::run(app_state).await
        }
    });

    let update_handle = tokio::spawn({
        let app_state = app_state.clone();
        async move {
            update::run(app_state).await
        }
    });

    let _ = tokio::join!(
        api_handle,
        update_handle
    );

    Ok(())
}
