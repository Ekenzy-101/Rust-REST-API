use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, time::Duration},
    get, post, web,
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    AppState, config,
    entity::{error::AppError, user},
};

#[derive(Debug, Validate, Deserialize)]
struct LoginUserRequest {
    #[validate(email(message = "Email must be valid"))]
    email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    password: String,
}

#[post("/auth/login")]

pub async fn login_user(
    state: web::Data<AppState>,
    body: web::Json<LoginUserRequest>,
) -> Result<HttpResponse, AppError> {
    let AppState { repo, auth } = state.get_ref();
    if let Err(err) = body.validate() {
        return Err(AppError::Validation(err));
    }

    let user = repo.get_user_by_email(body.email.clone()).await?;
    auth.verify_password(&body.password, &user.password)?;
    let token = auth.generate_access_token(&user)?;

    let res = HttpResponse::Ok()
        .cookie(
            Cookie::build(config::ACCESS_TOKEN_COOKIE_NAME, token)
                .http_only(true)
                .max_age(Duration::seconds(config::ACCESS_TOKEN_TTL_IN_SECONDS))
                .path("/")
                .secure(true)
                .finish(),
        )
        .json(user.set_password("".to_string()));
    Ok(res)
}

#[derive(Debug, Validate, Deserialize)]
struct RegisterUserRequest {
    #[validate(length(min = 1, message = "Name must not be empty"))]
    name: String,
    #[validate(email(message = "Email must be valid"))]
    email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    password: String,
}

#[post("/auth/register")]

pub async fn register_user(
    state: web::Data<AppState>,
    body: web::Json<RegisterUserRequest>,
) -> Result<HttpResponse, AppError> {
    let AppState { repo, auth } = state.get_ref();
    if let Err(err) = body.validate() {
        return Err(AppError::Validation(err));
    }

    let password = auth.hash_password(&body.password)?;
    let mut user = user::Model {
        email: body.email.clone(),
        name: body.name.clone(),
        password,
        ..Default::default()
    };
    user = repo.create_user(user).await?;
    let token = auth.generate_access_token(&user)?;

    let res = HttpResponse::Ok()
        .cookie(
            Cookie::build(config::ACCESS_TOKEN_COOKIE_NAME, token)
                .http_only(true)
                .max_age(Duration::seconds(config::ACCESS_TOKEN_TTL_IN_SECONDS))
                .path("/")
                .secure(true)
                .finish(),
        )
        .json(user.set_password("".to_string()));
    Ok(res)
}

#[get("/auth/me")]
pub async fn get_auth_user(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let AppState { repo, auth } = state.get_ref();
    let mut user = auth.extract_auth_user(req)?;
    user = repo.get_user_by_id(user.id).await?;

    let res = HttpResponse::Ok().json(user.set_password("".to_string()));
    Ok(res)
}
