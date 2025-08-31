mod others;
mod post;
mod user;

pub use others::*;
pub use post::*;
pub use user::*;

use crate::{adapter::Auth, config, entity::user::Model};
use actix_web::{HttpRequest, web};
use anyhow::{Result, anyhow};

fn extract_auth_user(auth: web::Data<Auth>, req: HttpRequest) -> Result<Model> {
    let result = req.cookie(config::ACCESS_TOKEN_COOKIE_NAME);
    if result.is_none() {
        return Err(anyhow!(
            "Cookie '{}' not found",
            config::ACCESS_TOKEN_COOKIE_NAME
        ));
    }

    return auth.verify_access_token(result.unwrap().value().into());
}
