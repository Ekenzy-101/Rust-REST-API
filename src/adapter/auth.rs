use std::sync::Arc;

use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config, entity::user};

#[derive(Debug, Clone, Copy)]
pub struct Auth {}

impl Auth {
    pub fn new() -> Arc<Self> {
        Arc::new(Auth {})
    }

    pub fn generate_access_token(&self, user: &user::Model) -> Result<String> {
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("primary".to_string());

        let now = Utc::now();
        let claims = Claims {
            aud: "web".to_string(),
            exp: (now + Duration::seconds(config::ACCESS_TOKEN_TTL_IN_SECONDS)).timestamp(),
            iat: now.timestamp(),
            iss: "api".to_string(),
            jti: Uuid::now_v7(),
            sub: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
        };

        let secret = config::access_token_secret();
        let key = EncodingKey::from_secret(secret.as_bytes());
        match encode(&header, &claims, &key) {
            Ok(token) => Ok(token),
            Err(err) => Err(anyhow!("{}", err)),
        }
    }

    pub fn verify_access_token(&self, token: &str) -> Result<user::Model> {
        let secret = config::access_token_secret();
        let key = DecodingKey::from_secret(secret.as_bytes());

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["web"]);
        validation.set_issuer(&["api"]);
        validation.set_required_spec_claims(&["aud", "exp", "iss", "sub"]);
        match decode::<Claims>(token, &key, &validation) {
            Ok(data) => Ok(user::Model {
                id: data.claims.sub,
                email: data.claims.email.clone(),
                name: data.claims.name.clone(),
                ..Default::default()
            }),
            Err(err) => Err(anyhow!("{}", err)),
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        match Argon2::default().hash_password(password.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(err) => Err(anyhow!("{}", err)),
        }
    }

    pub fn verify_password(&self, password: &str, password_hash: &str) -> Result<()> {
        match PasswordHash::new(password_hash) {
            Ok(hash) => match Argon2::default().verify_password(password.as_bytes(), &hash) {
                Ok(_) => Ok(()),
                Err(err) => Err(anyhow!("{}", err)),
            },
            Err(err) => Err(anyhow!("{}", err)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String, // Audience
    exp: i64,    // Expiration Time (as UTC timestamp seconds)
    iat: i64,    // Issued At
    iss: String, // Issuer
    jti: Uuid,   // JWT ID
    sub: Uuid,   // Subject (user id)
    email: String,
    name: String,
}
