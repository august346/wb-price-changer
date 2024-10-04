mod router;
mod middlewares;
mod ping;

use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use crate::state::AppState;
use crate::utils;

pub async fn run(app_state: Arc<AppState>) -> Result<(), String> {
    let mut router = router::get_router(app_state);

    if Some("1") == utils::get_env_or("DEBUG", "0".to_string()).ok().as_deref() {
        info!("will be allowed any cors");
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        router = router.layer(cors);
    }

    let host = utils::get_env_var("HOST")?;
    let port = utils::get_env_var("PORT")?;
    let bind_address = format!("{}:{}", host, port);
    info!("Listening on {}", bind_address);
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .expect("Failed init listener");

    axum::serve(listener, router.into_make_service()).await.expect("Failed start serving");

    Ok(())
}