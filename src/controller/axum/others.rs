use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::{AppState, entity::error::AppError};

pub async fn check_health(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    state.repo.check_health().await?;
    let res = (StatusCode::OK, Json(json!({"status": "ok"}))).into_response();
    Ok(res)
}

pub async fn not_found() -> Result<Response, AppError> {
    Err(AppError::NotFound("Route doesn't exist".to_string()))
}
