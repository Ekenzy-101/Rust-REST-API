use actix_web::{
    body::BoxBody,
    cookie::Cookie,
    dev::ServiceResponse,
    http::{Method, StatusCode},
    test, web,
};
use pretty_assertions::assert_eq;
use serde_json::*;

use crate::{
    AppState, adapter, config,
    controller::actix::*,
    entity::{error::AppErrorResponse, user},
    repository,
};

#[derive(Clone)]
struct Params {
    body: Value,
    method: Method,
    state: web::Data<AppState>,
    token: String,
    uri: &'static str,
}

async fn execute(
    Params {
        body,
        method,
        state,
        token,
        uri,
    }: Params,
) -> ServiceResponse<BoxBody> {
    let cookie = Cookie::new(config::ACCESS_TOKEN_COOKIE_NAME, token);
    let mut req = test::TestRequest::with_uri(uri).cookie(cookie);
    req = match method {
        Method::POST | Method::PUT => req.method(method).set_json(body),
        _ => req.method(method),
    };

    let app = test::init_service(new_app(state)).await;
    let res = test::call_service(&app, req.to_request()).await;
    res.map_into_boxed_body()
}

fn has_access_token(c: Cookie) -> bool {
    c.name().eq(config::ACCESS_TOKEN_COOKIE_NAME)
}

#[tokio::test]
async fn test_login_user() {
    env_logger::init();
    log::info!("It should return 201 if inputs are valid");
    let auth = adapter::Auth::new();
    let repo = repository::new().await.unwrap();
    let state = web::Data::new(AppState {
        auth: auth.clone(),
        repo: repo.clone(),
    });
    let body = RegisterUserRequest {
        email: "ekeneonyekaba@gmail.com".to_string(),
        name: "Ekene Onyekaba".to_string(),
        password: "testing@123".to_string(),
    };
    let mut params = Params {
        body: json!(body),
        method: Method::POST,
        token: "".to_string(),
        uri: "/auth/register",
        state,
    };

    let res = execute(params.clone()).await;
    assert_eq!(res.status(), StatusCode::CREATED);
    assert!(res.response().cookies().any(has_access_token));

    log::info!("It should return 200 if inputs are valid");
    params.body = json!(LoginUserRequest {
        email: body.email.clone(),
        password: body.password,
    });
    params.uri = "/auth/login";
    let res = execute(params.clone()).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.response().cookies().any(has_access_token));

    let user: user::Model = test::read_body_json(res).await;
    assert_eq!(user.email, body.email);
    assert_eq!(user.name, body.name);
    assert!(!user.id.is_nil());
    assert!(user.password.is_empty());

    log::info!("It should return 404 if user doesn't exist");
    let result = repo.delete_user_by_id(user.id).await;
    assert!(result.is_ok());

    let res = execute(params.clone()).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let error: AppErrorResponse = test::read_body_json(res).await;
    assert_eq!(error.code, "NOT_FOUND".to_string());
    assert_eq!(error.status, StatusCode::NOT_FOUND.as_u16());
    assert!(error.details.is_null());
    assert!(!error.message.is_empty());

    log::info!("It should return 422 if inputs are invalid");
    params.body = json!(LoginUserRequest {
        email: "invalid".to_string(),
        password: "".to_string(),
    });
    let res = execute(params.clone()).await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let error: AppErrorResponse = test::read_body_json(res).await;
    assert_eq!(error.code, "UNPROCESSABLE_ENTITY".to_string());
    assert_eq!(error.status, StatusCode::UNPROCESSABLE_ENTITY.as_u16());
    assert!(error.details.is_object());
    assert!(!error.message.is_empty());

    let result = repo.clear().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_user() {
    log::info!("It should return 201 if inputs are valid");
    let auth = adapter::Auth::new();
    let repo = repository::new().await.unwrap();
    let state = web::Data::new(AppState {
        auth: auth.clone(),
        repo: repo.clone(),
    });
    let body = RegisterUserRequest {
        email: "ekeneonyekaba@gmail.com".to_string(),
        name: "Ekene Onyekaba".to_string(),
        password: "testing@123".to_string(),
    };
    let mut params = Params {
        body: json!(body),
        method: Method::POST,
        token: "".to_string(),
        uri: "/auth/register",
        state,
    };

    let res = execute(params.clone()).await;
    assert_eq!(res.status(), StatusCode::CREATED);
    assert!(res.response().cookies().any(has_access_token));

    let user: user::Model = test::read_body_json(res).await;
    assert_eq!(user.email, body.email);
    assert_eq!(user.name, body.name);
    assert!(!user.id.is_nil());
    assert!(user.password.is_empty());

    log::info!("It should return 409 if user exists");
    let res = execute(params.clone()).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);

    let error: AppErrorResponse = test::read_body_json(res).await;
    assert_eq!(error.code, "CONFLICT".to_string());
    assert_eq!(error.status, StatusCode::CONFLICT.as_u16());
    assert!(error.details.is_null());
    assert!(!error.message.is_empty());

    log::info!("It should return 422 if inputs are invalid");
    params.body = json!(RegisterUserRequest {
        name: "".to_string(),
        email: "invalid".to_string(),
        password: "".to_string(),
    });
    let res = execute(params.clone()).await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let error: AppErrorResponse = test::read_body_json(res).await;
    assert_eq!(error.code, "UNPROCESSABLE_ENTITY".to_string());
    assert_eq!(error.status, StatusCode::UNPROCESSABLE_ENTITY.as_u16());
    assert!(error.details.is_object());
    assert!(!error.message.is_empty());

    let result = repo.clear().await;
    assert!(result.is_ok());
}
