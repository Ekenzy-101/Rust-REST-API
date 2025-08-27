use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

use crate::repository::Repository;

#[get("/health")]
pub async fn check_health(repo: web::Data<dyn Repository>) -> impl Responder {
    match repo.check_health().await  {
        Ok(_) => HttpResponse::Ok().json(json!({
            "status": "ok" 
        })),
        Err(err) => HttpResponse::ServiceUnavailable().json(json!({
            "message": err.to_string(),
        })),
    }    
}

