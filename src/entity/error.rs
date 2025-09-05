use actix_web::{HttpResponse, ResponseError, http::StatusCode};
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

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status: StatusCode;
        let message: &str;
        let mut details = json!(null);
        match self {
            AppError::Conflict(msg) => {
                status = StatusCode::CONFLICT;
                message = msg;
            }
            AppError::Forbidden(msg) => {
                status = StatusCode::FORBIDDEN;
                message = msg;
            }
            AppError::Internal { err, path } => {
                status = StatusCode::INTERNAL_SERVER_ERROR;
                message = "Something went wrong";
                println!("Err: {err} at path: {path}")
            }
            AppError::NotFound(msg) => {
                status = StatusCode::NOT_FOUND;
                message = msg;
            }
            AppError::Unauthorized(msg) => {
                status = StatusCode::UNAUTHORIZED;
                message = msg;
            }
            AppError::Validation(errors) => {
                status = StatusCode::UNPROCESSABLE_ENTITY;
                message = "Invalid request payload";
                details = json!(errors)
            }
        }

        HttpResponse::build(status).json(json!({
            "code": status.canonical_reason().map_or("UNKNOWN".into(), |s| s.replace(" ", "_").to_uppercase()),
            "message": message,
            "status": status.as_u16(),
            "details": details,
        }))
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
