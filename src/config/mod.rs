use std::env;

pub const ACCESS_TOKEN_COOKIE_NAME: &str = "access_token";
pub const ACCESS_TOKEN_TTL_IN_SECONDS: i64 = 60 * 60 * 24;
pub const COLLECTION_POSTS: &str = "posts";
pub const COLLECTION_USERS: &str = "users";

pub fn access_token_secret() -> String {
    return env::var("ACCESS_TOKEN_SECRET").unwrap_or(String::new());
}

pub fn database_name() -> String {
    return env::var("DATABASE_NAME").unwrap_or(String::new());
}

pub fn database_type() -> String {
    return env::var("DATABASE_TYPE").unwrap_or(String::new());
}

pub fn database_url() -> String {
    return env::var("DATABASE_URL").unwrap_or(String::new());
}

pub fn framework_type() -> String {
    return env::var("FRAMEWORK_TYPE").unwrap_or(String::new());
}

pub fn is_production() -> bool {
    return env::var("RUST_ENV").unwrap_or(String::from("development"))
        == String::from("production");
}

pub fn port() -> String {
    return env::var("PORT").unwrap_or(String::from("5000"));
}
