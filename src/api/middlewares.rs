use std::sync::Arc;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use crate::state::AppState;

pub async fn get_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(api_key_header) = request.headers().get("Authorization") {
        if let Ok(api_key) = api_key_header.to_str() {
            if let Ok(supplier) = state.get_supplier(api_key).await {
                request.extensions_mut().insert(supplier);
                return Ok(next.run(request).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)?
}