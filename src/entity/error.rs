use actix_web::{HttpResponse, ResponseError, http::StatusCode as ActixStatusCode};
use axum::{
    http::StatusCode as AxumStatusCode,
    response::{IntoResponse, Response},
};
use mongodb::error::{ErrorKind, WriteFailure};
use sea_orm::SqlErr;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;
use validator::ValidationErrors;

use crate::config;

#[derive(Clone, Debug, Error)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppErrorResponse {
    pub code: String,
    pub message: String,
    pub status: u16,
    pub details: Value,
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
                message = "Something went wrong".to_string();
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
                message = "Invalid request payload".to_string();
                details = json!(errors);
            }
        }

        let body = AppErrorResponse {
            code: status
                .canonical_reason()
                .map_or("UNKNOWN".to_string(), |s| {
                    s.replace(" ", "_").to_uppercase()
                }),
            message,
            status: status.as_u16(),
            details,
        };
        (status, axum::Json(body)).into_response()
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let mut details = json!(null);
        let message: String;
        let status: ActixStatusCode;
        match self {
            AppError::Conflict(msg) => {
                message = msg.to_string();
                status = ActixStatusCode::CONFLICT;
            }
            AppError::Forbidden(msg) => {
                message = msg.to_string();
                status = ActixStatusCode::FORBIDDEN;
            }
            AppError::Internal { err, path } => {
                log::error!("Err: {err} at path: {path}");
                message = "Something went wrong".to_string();
                status = ActixStatusCode::INTERNAL_SERVER_ERROR;
            }
            AppError::NotFound(msg) => {
                message = msg.to_string();
                status = ActixStatusCode::NOT_FOUND;
            }
            AppError::Unauthorized(msg) => {
                message = msg.to_string();
                status = ActixStatusCode::UNAUTHORIZED;
            }
            AppError::Validation(errors) => {
                details = json!(errors);
                message = "Invalid request payload".to_string();
                status = ActixStatusCode::UNPROCESSABLE_ENTITY;
            }
        }

        let body = AppErrorResponse {
            code: status
                .canonical_reason()
                .map_or("UNKNOWN".to_string(), |s| {
                    s.replace(" ", "_").to_uppercase()
                }),
            message,
            status: status.as_u16(),
            details,
        };
        HttpResponse::build(status).json(body)
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        match err.kind.as_ref() {
            ErrorKind::Write(WriteFailure::WriteError(write_err)) => {
                if write_err.code == 11000 && write_err.message.contains(config::COLLECTION_USERS) {
                    return AppError::Conflict(format!("User already exists"));
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
