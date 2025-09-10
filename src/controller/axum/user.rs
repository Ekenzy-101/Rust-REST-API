use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite, time::Duration};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    AppState, config,
    entity::{error::AppError, user},
};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LoginUserRequest {
    #[validate(email(message = "Email must be valid"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

pub async fn login_user(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<LoginUserRequest>,
) -> Result<Response<Body>, AppError> {
    let AppState { repo, auth } = state.as_ref();
    if let Err(err) = body.validate() {
        return Err(AppError::Validation(err));
    }

    let user = repo.get_user_by_email(body.email.clone()).await?;
    auth.verify_password(&body.password, &user.password)?;
    let token = auth.generate_access_token(&user)?;

    let cookie = Cookie::build((config::ACCESS_TOKEN_COOKIE_NAME, token))
        .http_only(true)
        .max_age(Duration::seconds(config::ACCESS_TOKEN_TTL_IN_SECONDS))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(config::is_production());
    let res = (
        StatusCode::OK,
        jar.add(cookie),
        Json(user.set_password("".to_string())),
    )
        .into_response();
    Ok(res)
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct RegisterUserRequest {
    #[validate(length(min = 1, message = "Name must not be empty"))]
    pub name: String,
    #[validate(email(message = "Email must be valid"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

pub async fn register_user(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<RegisterUserRequest>,
) -> Result<Response, AppError> {
    let AppState { repo, auth } = state.as_ref();
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

    let cookie = Cookie::build((config::ACCESS_TOKEN_COOKIE_NAME, token))
        .http_only(true)
        .max_age(Duration::seconds(config::ACCESS_TOKEN_TTL_IN_SECONDS))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(config::is_production());
    let res = (
        StatusCode::CREATED,
        jar.add(cookie),
        Json(user.set_password("".to_string())),
    )
        .into_response();
    Ok(res)
}

pub async fn get_auth_user(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Response, AppError> {
    let AppState { repo, auth } = state.as_ref();
    let mut user = auth.extract_user_from_axum(jar)?;
    user = repo.get_user_by_id(user.id).await?;

    let res = (StatusCode::OK, Json(user.set_password("".to_string()))).into_response();
    Ok(res)
}
