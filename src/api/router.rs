use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use reqwest::StatusCode;
use serde::Deserialize;
use crate::api::middlewares::get_auth;
use crate::api::ping::ping;
use crate::db::product::Product;
use crate::db::supplier::Supplier;
use crate::state::AppState;
use crate::wb::calculate_and_set_price;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping))
        .route("/create_api_key", post(create_api_key))
        .route(
            "/update_price",
            post(update_price).layer(
                middleware::from_fn_with_state(app_state.clone(), get_auth)
            ),
        )
        .route(
            "/set_wb_jwt",
            post(set_wb_jwt).layer(
                middleware::from_fn_with_state(app_state.clone(), get_auth)
            ),
        )
        .with_state(app_state)
}

async fn update_price(
    State(state): State<Arc<AppState>>,
    Extension(supplier): Extension<Supplier>,
    Json(input): Json<Product>,
) -> Result<impl IntoResponse, StatusCode> {
    state.task_manager.remove_task(input.id).await;

    let wb_jwt = match supplier.wb_jwt {
        None => return Ok(Json("Need set JWT".to_string())),
        Some(token) => token,
    };

    match calculate_and_set_price(input.id, input.price, &wb_jwt).await {
        Ok((new_price, handle)) => {
            state.task_manager.add_task(input.id, handle).await;
            Ok(Json(format!("New price for ID {} set to {}", input.id, new_price)))
        },
        Err(err_msg) => Ok(Json(err_msg)),
    }
}

async fn create_api_key(State(state): State<Arc<AppState>>) -> Json<String> {
    let supplier = match state.create_supplier().await {
        Ok(s) => s,
        Err(err) => return Json(err)
    };

    Json(format!("Created ApiKey: {}", supplier.api_key))
}

#[derive(Deserialize)]
struct SetWbJwt {
    jwt: String
}

async fn set_wb_jwt(
    State(state): State<Arc<AppState>>,
    Extension(supplier): Extension<Supplier>,
    Json(input): Json<SetWbJwt>,
) -> Json<String> {
    if let Err(err) = state.set_wb_jwt(&supplier.api_key, &input.jwt).await {
        return Json(format!("Failed set wb jwt: {}", err))
    }

    Json("Set wb jwt success".to_string())
}
