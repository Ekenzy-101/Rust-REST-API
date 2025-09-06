use actix_web::{HttpRequest, HttpResponse, delete, get, post, put, web};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    entity::{error::AppError, post},
};

#[derive(Debug, Deserialize, Validate)]
struct CreatePostRequest {
    #[validate(length(min = 1, message = "Content must not be empty"))]
    content: String,
    #[validate(length(min = 1, message = "Title must not be empty"))]
    title: String,
}

#[post("/posts")]
pub async fn create_post(
    body: web::Json<CreatePostRequest>,
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let AppState { repo, auth } = state.get_ref();
    let user = auth.extract_user_from_actix(req)?;
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

    let res = HttpResponse::Created().json(post.set_user(user));
    Ok(res)
}

#[delete("/posts/{id}")]
pub async fn delete_post(
    id: web::Path<Uuid>,
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let AppState { repo, auth } = state.get_ref();
    let user = auth.extract_user_from_actix(req)?;

    let post = repo.get_post_by_id(id.into_inner()).await?;
    if post.user.unwrap().id != user.id {
        return Err(AppError::Forbidden("You can't delete this post".into()));
    }

    repo.delete_post_by_id(post.id).await?;
    let res = HttpResponse::Ok().json(json!({"message": "Success"}));
    Ok(res)
}

#[get("/posts")]
pub async fn get_posts(
    query: web::Query<post::Pagination>,
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let AppState { repo, auth } = state.get_ref();
    auth.extract_user_from_actix(req)?;
    if let Err(err) = query.validate() {
        return Err(AppError::Validation(err));
    }

    let posts = repo.get_posts(query.into_inner()).await?;
    let res = HttpResponse::Ok().json(json!(posts));
    Ok(res)
}

#[get("/posts/{id}")]
pub async fn get_post(
    id: web::Path<Uuid>,
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let AppState { repo, auth } = state.get_ref();
    auth.extract_user_from_actix(req)?;
    let post = repo.get_post_by_id(id.into_inner()).await?;

    let res = HttpResponse::Ok().json(post);
    Ok(res)
}

#[derive(Debug, Deserialize, Validate)]
struct UpdatePostRequest {
    #[validate(length(min = 1, message = "Content must not be empty"))]
    content: String,
    #[validate(length(min = 1, message = "Title must not be empty"))]
    title: String,
}

#[put("/posts/{id}")]
pub async fn update_post(
    body: web::Json<UpdatePostRequest>,
    id: web::Path<Uuid>,
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let AppState { repo, auth } = state.get_ref();
    let user = auth.extract_user_from_actix(req)?;
    if let Err(err) = body.validate() {
        return Err(AppError::Validation(err));
    }

    let mut post = repo.get_post_by_id(id.into_inner()).await?;
    post = post::Model {
        content: body.content.clone(),
        title: body.title.clone(),
        updated_at: chrono::Utc::now(),
        ..post
    };

    if post.user.clone().unwrap().id != user.id {
        return Err(AppError::Forbidden("You can't update this post".into()));
    }

    post = repo.update_post(post).await?;
    let res = HttpResponse::Ok().json(post);
    Ok(res)
}
