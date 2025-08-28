use actix_web::{cookie::{time::Duration, Cookie}, get, http::StatusCode, post, web, HttpRequest, HttpResponse, HttpResponseBuilder, Responder};
use serde::Deserialize;
use serde_json::json;
use validator::{Validate};

use crate::{adapter::Auth, config, entity::user, repository::Repository};

#[derive(Debug, Validate, Deserialize)]
struct LoginUserRequest {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
}

#[post("/auth/login")]

pub async fn login_user(auth: web::Data<Auth>, body: web::Json<LoginUserRequest>, repo: web::Data<dyn Repository>, ) -> impl Responder {
    if let Err(err) = body.validate() {
        return HttpResponse::BadRequest().json(json!({
            "message": err.to_string(),
        }));
    }
    
    let result = repo.get_user_by_email(body.email.clone()).await;
    if let Err(err) = result {
        let value = json!({"message": err.to_string()});
        let mut status = StatusCode::INTERNAL_SERVER_ERROR;
        if err.to_string().contains("not found") {
            status = StatusCode::NOT_FOUND
        }
        
        return HttpResponseBuilder::new(status).json(value);
    }

    let mut user = result.unwrap();
    let result = auth.verify_password(&body.password, &user.password);
    if let Err(err) = result {
        return HttpResponse::InternalServerError().json(json!({
            "message": err.to_string(),
        }));
    }

    let result = auth.generate_access_token(&user);
    if let Err(err) = result {
        return HttpResponse::InternalServerError().json(json!({
            "message": err.to_string(),
        }));
    }

    HttpResponse::Ok()
    .cookie(
        Cookie::build(config::ACCESS_TOKEN_COOKIE_NAME, result.unwrap())
            .http_only(true)
            .max_age(Duration::seconds(config::ACCESS_TOKEN_TTL_IN_SECONDS))
            .path("/")
            .secure(true)
            .finish()).
    json(user.set_password("".to_string()))  
}

#[derive(Debug, Validate, Deserialize)]
struct RegisterUserRequest {
    #[validate(length(min = 1))]
    name: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
}

#[post("/auth/register")]

pub async fn register_user(auth: web::Data<Auth>, body: web::Json<RegisterUserRequest>, repo: web::Data<dyn Repository>, ) -> impl Responder {
    if let Err(err) = body.validate() {
        return HttpResponse::BadRequest().json(json!({
            "message": err.to_string(),
        }));
    }
    
    let result = auth.hash_password(&body.password);
    if let Err(err) = result {
        return HttpResponse::InternalServerError().json(json!({
            "message": err.to_string(),
        }));
    }

    let user = user::Model {
        email: body.email.clone(),
        name: body.name.clone(),
        password: result.unwrap(),
        ..Default::default()
    };
    let result = repo.create_user(user).await;
    if let Err(err) = result {
        return HttpResponse::InternalServerError().json(json!({
            "message": err.to_string(),
        }));
    }

    let mut user = result.unwrap();
    let result = auth.generate_access_token(&user);
    if let Err(err) = result {
        return HttpResponse::InternalServerError().json(json!({
            "message": err.to_string(),
        }));
    }

    HttpResponse::Ok()
    .cookie(
        Cookie::build(config::ACCESS_TOKEN_COOKIE_NAME, result.unwrap())
            .http_only(true)
            .max_age(Duration::seconds(config::ACCESS_TOKEN_TTL_IN_SECONDS))
            .path("/")
            .secure(true)
            .finish()).
    json(user.set_password("".to_string()))  
}

#[get("/auth/me")]
pub async fn get_auth_user(auth: web::Data<Auth>, repo: web::Data<dyn Repository>, req: HttpRequest) -> impl Responder {
    let result = req.cookie(config::ACCESS_TOKEN_COOKIE_NAME);
    if result.is_none()  {
        return HttpResponse::Unauthorized().json(json!({
            "message": format!("Cookie '{}' not found", config::ACCESS_TOKEN_COOKIE_NAME),
        }));
    }

    let result = auth.verify_access_token(result.unwrap().value().into());
    if let Err(err) = result {
        return HttpResponse::Unauthorized().json(json!({
            "message": err.to_string(),
        }));
    }

    match repo.get_user_by_id(result.unwrap().id).await  {
        Ok(mut user) => HttpResponse::Ok().json(user.set_password("".to_string())),
        Err(err) => {
            let value = json!({"message": err.to_string()});
            let mut status = StatusCode::INTERNAL_SERVER_ERROR;
            if err.to_string().contains("not found") {
                status = StatusCode::NOT_FOUND
            }
            
            return HttpResponseBuilder::new(status).json(value);
        }
    }    
}
