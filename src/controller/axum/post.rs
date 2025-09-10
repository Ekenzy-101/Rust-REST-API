use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    entity::{error::AppError, post},
};

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, message = "Content must not be empty"))]
    content: String,
    #[validate(length(min = 1, message = "Title must not be empty"))]
    title: String,
}

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<CreatePostRequest>,
) -> Result<Response, AppError> {
    let AppState { repo, auth } = state.as_ref();
    let user = auth.extract_user_from_axum(jar)?;
    if let Err(err) = body.validate() {
        return Err(AppError::Validation(err));
    }

    let mut post = post::Model {
        content: body.content.clone(),
        title: body.title.clone(),
        user_id: user.id,
        ..Default::default()
    };
    post = repo.create_post(post).await?;

    let res = (StatusCode::CREATED, Json(post.set_user(user))).into_response();
    Ok(res)
}

pub async fn delete_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    jar: CookieJar,
) -> Result<Response, AppError> {
    let AppState { repo, auth } = state.as_ref();
    let user = auth.extract_user_from_axum(jar)?;

    let post = repo.get_post_by_id(id).await?;
    if post.user.unwrap().id != user.id {
        return Err(AppError::Forbidden("You can't delete this post".to_string()));
    }

    repo.delete_post_by_id(post.id).await?;
    let res = (Json(json!({"message": "Success"}).to_string())).into_response();
    Ok(res)
}

pub async fn get_posts(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(query): Query<post::Pagination>,
) -> Result<Response, AppError> {
    let AppState { repo, auth } = state.as_ref();
    auth.extract_user_from_axum(jar)?;
    if let Err(err) = query.validate() {
        return Err(AppError::Validation(err));
    }

    let posts = repo.get_posts(query).await?;
    let res = (Json(posts)).into_response();
    Ok(res)
}

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let AppState { repo, auth } = state.as_ref();
    auth.extract_user_from_axum(jar)?;
    let post = repo.get_post_by_id(id).await?;

    let res = (Json(post)).into_response();
    Ok(res)
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePostRequest {
    #[validate(length(min = 1, message = "Content must not be empty"))]
    content: String,
    #[validate(length(min = 1, message = "Title must not be empty"))]
    title: String,
}

pub async fn update_post(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePostRequest>,
) -> Result<Response, AppError> {
    let AppState { repo, auth } = state.as_ref();
    let user = auth.extract_user_from_axum(jar)?;
    if let Err(err) = body.validate() {
        return Err(AppError::Validation(err));
    }

    let mut post = repo.get_post_by_id(id).await?;
    post = post::Model {
        content: body.content.clone(),
        title: body.title.clone(),
        updated_at: chrono::Utc::now(),
        ..post
    };

    if post.user.clone().unwrap().id != user.id {
        return Err(AppError::Forbidden("You can't update this post".to_string()));
    }

    post = repo.update_post(post).await?;
    let res = (Json(post)).into_response();
    Ok(res)
}
