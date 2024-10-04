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
    utils::get_env_var("JWT")?;

    tracing_subscriber::fmt().json()
        .with_max_level(Level::ERROR)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app_state = Arc::new(AppState::setup_app_state()
        .await
        .expect("Failed to build AppState"));
    app_state.run_migrations().await?;

    // wb::get_supplier_catalog(688305, None, None).await?;

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

    let _ = tokio::join!(api_handle, update_handle);

    Ok(())
}
