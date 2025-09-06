use actix_web::{HttpResponse, ResponseError, http::StatusCode as ActixStatusCode};
use axum::{
    http::StatusCode as AxumStatusCode,
    response::{IntoResponse, Response},
};
use mongodb::error::{ErrorKind, WriteFailure};
use sea_orm::SqlErr;
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

use crate::config;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Internal error: {err} at path: {path}")]
    Internal { err: String, path: String },
    #[error("NotFound: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Validation: {0}")]
    Validation(ValidationErrors),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut details = json!(null);
        let status: AxumStatusCode;
        let message: String;
        match self {
            AppError::Conflict(msg) => {
                status = AxumStatusCode::CONFLICT;
                message = msg;
            }
            AppError::Forbidden(msg) => {
                status = AxumStatusCode::FORBIDDEN;
                message = msg;
            }
            AppError::Internal { err, path } => {
                status = AxumStatusCode::INTERNAL_SERVER_ERROR;
                message = "Something went wrong".into();
                log::error!("Err: {err} at path: {path}");
            }
            AppError::NotFound(msg) => {
                status = AxumStatusCode::NOT_FOUND;
                message = msg;
            }
            AppError::Unauthorized(msg) => {
                status = AxumStatusCode::UNAUTHORIZED;
                message = msg;
            }
            AppError::Validation(errors) => {
                status = AxumStatusCode::UNPROCESSABLE_ENTITY;
                message = "Invalid request payload".into();
                details = json!(errors);
            }
        }

        let body = json!({
            "code": status.canonical_reason().map_or("UNKNOWN".into(), |s| s.replace(" ", "_").to_uppercase()),
            "message": message,
            "status": status.as_u16(),
            "details": details,
        });
        (status, axum::Json(body)).into_response()
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let mut details = json!(null);
        let message: &str;
        let status: ActixStatusCode;
        match self {
            AppError::Conflict(msg) => {
                message = msg;
                status = ActixStatusCode::CONFLICT;
            }
            AppError::Forbidden(msg) => {
                message = msg;
                status = ActixStatusCode::FORBIDDEN;
            }
            AppError::Internal { err, path } => {
                log::error!("Err: {err} at path: {path}");
                message = "Something went wrong";
                status = ActixStatusCode::INTERNAL_SERVER_ERROR;
            }
            AppError::NotFound(msg) => {
                message = msg;
                status = ActixStatusCode::NOT_FOUND;
            }
            AppError::Unauthorized(msg) => {
                message = msg;
                status = ActixStatusCode::UNAUTHORIZED;
            }
            AppError::Validation(errors) => {
                details = json!(errors);
                message = "Invalid request payload";
                status = ActixStatusCode::UNPROCESSABLE_ENTITY;
            }
        }

        let body = json!({
            "code": status.canonical_reason().map_or("UNKNOWN".into(), |s| s.replace(" ", "_").to_uppercase()),
            "message": message,
            "status": status.as_u16(),
            "details": details,
        });
        HttpResponse::build(status).json(body)
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        match err.kind.as_ref() {
            ErrorKind::Write(WriteFailure::WriteError(write_err)) => {
                if let Some(details) = &write_err.details
                    && write_err.code == 11000
                    && details
                        .get_str("ns")
                        .unwrap()
                        .contains(config::COLLECTION_USERS)
                {
                    let email = details
                        .get_document("keyValue")
                        .unwrap()
                        .get_str("email")
                        .unwrap();
                    return AppError::Conflict(format!("User '{email}' already exists"));
                }

                AppError::Internal {
                    err: err.to_string(),
                    path: format!("{}:{}:{}", file!(), line!(), column!()),
                }
            }
            _ => AppError::Internal {
                err: err.to_string(),
                path: format!("{}:{}:{}", file!(), line!(), column!()),
            },
        }
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        if let Some(sql_err) = err.sql_err()
            && let SqlErr::UniqueConstraintViolation(msg) = sql_err
            && msg.contains(config::COLLECTION_USERS)
        {
            return AppError::Conflict(format!("User already exists"));
        }

        AppError::Internal {
            err: err.to_string(),
            path: format!("{}:{}:{}", file!(), line!(), column!()),
        }
    }
}
