use actix_web::{HttpResponse, get, web};
use serde_json::json;

use crate::{AppState, entity::error::AppError};

#[get("/health")]
pub async fn check_health(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    state.repo.check_health().await?;
    let res = HttpResponse::Ok().json(json!({"status": "ok"}));
    Ok(res)
}
