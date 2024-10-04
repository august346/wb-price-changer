use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use crate::api::error::AppError;
use crate::api::middlewares::get_auth;
use crate::api::ping::ping;
use crate::db::product::Product;
use crate::db::supplier::Supplier;
use crate::state::AppState;
use crate::wb::calculate_and_set_price;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    let protected_routes = Router::new()
        .route("/set_wb_jwt", post(set_wb_jwt))
        .route("/update_price", post(update_price))
        .layer(middleware::from_fn_with_state(app_state.clone(), get_auth));

    Router::new()
        .route("/ping", get(ping))
        .route("/create_api_key", post(create_api_key))
        .nest("/", protected_routes)
        .with_state(app_state)
}

#[derive(Serialize)]
struct PriceSet {
    products: Vec<Product>
}

async fn update_price(
    State(state): State<Arc<AppState>>,
    Extension(supplier): Extension<Supplier>,
    Json(input): Json<Product>,
) -> Result<impl IntoResponse, AppError> {
    let task_id = 0;    // TODO wb_id

    state.task_manager.remove_task(task_id).await;

    let wb_jwt = supplier.wb_jwt
        .ok_or_else(|| AppError::NoPermission("Need set JWT".to_string()))?;

    match calculate_and_set_price(&wb_jwt, vec![Product::new(input.id, input.price)]).await {
        Ok((products, handle)) => {
            state.task_manager.add_task(task_id, handle).await;
            Ok(Json(PriceSet { products }))
        },
        Err(err_msg) => Err(AppError::unexpected(&err_msg)),
    }
}

async fn create_api_key(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let supplier = state.create_supplier()
        .await
        .map_err(|err| AppError::unexpected(&err))?;

    Ok((StatusCode::CREATED, supplier.api_key))
}

#[derive(Deserialize)]
struct SetWbJwt {
    jwt: String
}

#[derive(Serialize)]
struct Ok {
    ok: bool
}

async fn set_wb_jwt(
    State(state): State<Arc<AppState>>,
    Extension(supplier): Extension<Supplier>,
    Json(input): Json<SetWbJwt>,
) -> Result<impl IntoResponse, AppError> {
    state.set_wb_jwt(&supplier.api_key, &input.jwt)
        .await
        .map_err(|err| AppError::unexpected(&err))?;

    Ok(Json(Ok { ok: true }))
}
