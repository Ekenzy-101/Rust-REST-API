use actix_web::{
    HttpRequest, HttpResponse, HttpResponseBuilder, Responder, delete, get, http::StatusCode, post,
    put, web,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{adapter::Auth, controller::extract_auth_user, entity::post, repository::Repository};

#[derive(Debug, Deserialize, Validate)]
struct CreatePostRequest {
    #[validate(length(min = 1))]
    content: String,
    #[validate(length(min = 1))]
    title: String,
}

#[post("/posts")]
pub async fn create_post(
    auth: web::Data<Auth>,
    body: web::Json<CreatePostRequest>,
    repo: web::Data<dyn Repository>,
    req: HttpRequest,
) -> impl Responder {
    let result = extract_auth_user(auth, req);
    if let Err(err) = result {
        return HttpResponse::Unauthorized().json(json!({
            "message": err.to_string(),
        }));
    }

    let auth_user = result.unwrap();
    if let Err(err) = body.validate() {
        return HttpResponse::BadRequest().json(json!({
            "message": err.to_string(),
        }));
    }

    let post = post::Model {
        content: body.content.clone(),
        title: body.title.clone(),
        user_id: auth_user.id,
        ..Default::default()
    };
    match repo.create_post(post).await {
        Ok(post) => HttpResponse::Created().json(post.set_user(auth_user)),
        Err(err) => HttpResponse::ServiceUnavailable().json(json!({
            "message": err.to_string(),
        })),
    }
}

#[delete("/posts/{id}")]
pub async fn delete_post(
    auth: web::Data<Auth>,
    id: web::Path<Uuid>,
    repo: web::Data<dyn Repository>,
    req: HttpRequest,
) -> impl Responder {
    let result = extract_auth_user(auth, req);
    if let Err(err) = result {
        return HttpResponse::Unauthorized().json(json!({
            "message": err.to_string(),
        }));
    }

    let auth_user = result.unwrap();
    let result = repo.get_post_by_id(id.into_inner()).await;
    if let Err(err) = result {
        let value = json!({"message": err.to_string()});
        let mut status = StatusCode::INTERNAL_SERVER_ERROR;
        if err.to_string().contains("not found") {
            status = StatusCode::NOT_FOUND
        }
        return HttpResponseBuilder::new(status).json(value);
    }

    let post = result.unwrap();
    if post.user.unwrap().id != auth_user.id {
        return HttpResponse::Unauthorized().json(json!({
            "message": "User isn't allowed to delete this post",
        }));
    }

    match repo.delete_post_by_id(post.id).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Success",
        })),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "message": err.to_string()
        })),
    }
}

#[get("/posts")]
pub async fn get_posts(
    auth: web::Data<Auth>,
    query: web::Query<post::Pagination>,
    repo: web::Data<dyn Repository>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(err) = extract_auth_user(auth, req) {
        return HttpResponse::Unauthorized().json(json!({
            "message": err.to_string(),
        }));
    }

    if let Err(err) = query.validate() {
        return HttpResponse::BadRequest().json(json!({
            "message": err.to_string(),
        }));
    }

    match repo.get_posts(query.into_inner()).await {
        Ok(posts) => HttpResponse::Ok().json(json!(posts)),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "message": err.to_string()
        })),
    }
}

#[get("/posts/{id}")]
pub async fn get_post(
    auth: web::Data<Auth>,
    id: web::Path<Uuid>,
    repo: web::Data<dyn Repository>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(err) = extract_auth_user(auth, req) {
        return HttpResponse::Unauthorized().json(json!({
            "message": err.to_string(),
        }));
    }

    match repo.get_post_by_id(id.into_inner()).await {
        Ok(post) => HttpResponse::Ok().json(post),
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

#[derive(Debug, Deserialize, Validate)]
struct UpdatePostRequest {
    #[validate(length(min = 1))]
    content: String,
    #[validate(length(min = 1))]
    title: String,
}

#[put("/posts/{id}")]
pub async fn update_post(
    auth: web::Data<Auth>,
    body: web::Json<UpdatePostRequest>,
    id: web::Path<Uuid>,
    repo: web::Data<dyn Repository>,
    req: HttpRequest,
) -> impl Responder {
    let result = extract_auth_user(auth, req);
    if let Err(err) = result {
        return HttpResponse::Unauthorized().json(json!({
            "message": err.to_string(),
        }));
    }

    let auth_user = result.unwrap();
    if let Err(err) = body.validate() {
        return HttpResponse::BadRequest().json(json!({
            "message": err.to_string(),
        }));
    }

    let result = repo.get_post_by_id(id.into_inner()).await;
    if let Err(err) = result {
        let value = json!({"message": err.to_string()});
        let mut status = StatusCode::INTERNAL_SERVER_ERROR;
        if err.to_string().contains("not found") {
            status = StatusCode::NOT_FOUND
        }
        return HttpResponseBuilder::new(status).json(value);
    }

    let post = post::Model {
        content: body.content.clone(),
        title: body.title.clone(),
        updated_at: chrono::Utc::now(),
        ..result.unwrap()
    };
    if post.user.clone().unwrap().id != auth_user.id {
        return HttpResponse::Unauthorized().json(json!({
            "message": "User isn't allowed to update this post",
        }));
    }

    match repo.update_post(post).await {
        Ok(post) => HttpResponse::Ok().json(post),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "message": err.to_string()
        })),
    }
}
