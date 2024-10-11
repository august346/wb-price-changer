use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use crate::api::error::AppError;
use crate::api::middlewares::{get_auth, get_super};
use crate::api::ping::ping;
use crate::db::product::Product;
use crate::db::supplier::Supplier;
use crate::state::AppState;
use crate::utils;
use crate::wb::calculate_and_set_price;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    let protected_routes = Router::new()
        .route("/state", get(get_state))
        .route("/set_wb_jwt", post(set_wb_jwt))
        .route("/update_price", post(update_price))
        .layer(middleware::from_fn_with_state(app_state.clone(), get_auth));

    Router::new()
        .route("/ping", get(ping))
        .route("/create_api_key",
               post(create_api_key)
                   .layer(middleware::from_fn_with_state(app_state.clone(), get_super)))
        .nest("/", protected_routes)
        .with_state(app_state)
}

#[derive(Serialize)]
struct PriceSet {
    products: Vec<Product>,
}

async fn update_price(
    State(state): State<Arc<AppState>>,
    Extension(supplier): Extension<Supplier>,
    Json(input): Json<Product>,
) -> Result<impl IntoResponse, AppError> {
    state.task_manager.remove_task(input.id).await;

    let wb_jwt = supplier.wb_jwt
        .ok_or_else(|| AppError::NoPermission("Need set JWT".to_string()))?;

    match calculate_and_set_price(supplier.wb_id, &wb_jwt, vec![Product::new(input.id, input.price)]).await {
        Ok((supplier_id, products, handle)) => {
            if let Some(supplier_id) = supplier_id {
                state.set_wb_id(&supplier.api_key, supplier_id)
                    .await
                    .map_err(|err| AppError::unexpected(&err))?;
            }
            if let Some(handle) = handle {
                state.task_manager.add_task(input.id, handle).await;
            }
            let _ = state.add_goods(&supplier.api_key, &vec![input]).await;
            Ok(Json(PriceSet { products }))
        }
        Err(err_msg) => Err(AppError::unexpected(&err_msg)),
    }
}

async fn create_api_key(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let supplier = state.create_supplier()
        .await
        .map_err(|err| AppError::unexpected(&err))?;

    Ok((StatusCode::CREATED, supplier.api_key.to_string()))
}

#[derive(Deserialize)]
struct SetWbJwt {
    jwt: String,
}

#[derive(Serialize)]
struct Ok {
    ok: bool,
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

#[derive(Serialize)]
struct JwtState {
    expiry: usize
}

#[derive(Serialize)]
struct Products {
    current: usize,
    max: u32,
}

#[derive(Serialize)]
struct UserState {
    jwt: Option<JwtState>,
    products: Products
}

async fn get_state(
    State(state): State<Arc<AppState>>,
    Extension(supplier): Extension<Supplier>,
) -> Result<impl IntoResponse, AppError> {
    let jwt_expire_ts = match &supplier.wb_jwt {
        None => None,
        Some(jwt) => Some(utils::get_jwt_expire(jwt)
            .map_err(|err| AppError::unexpected(&err))?),
    };

    let current_monitored = state.count_by_apikey(&supplier.api_key)
        .await
        .map_err(|err| AppError::unexpected(&err))?;
    let max_monitored = 100;

    let us = UserState {
        jwt: match jwt_expire_ts {
            None => None,
            Some(expiry) => Some(JwtState{ expiry: expiry * 1000 })
        },
        products: Products{ current: current_monitored as usize, max: max_monitored }
    };

    Ok(Json(us))
}
