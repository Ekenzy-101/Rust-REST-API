use std::env;

pub const COLLECTION_POSTS: &str = "posts";
pub const COLLECTION_USERS: &str = "users";

pub fn database_name() -> String {
    return env::var("DATABASE_NAME").unwrap_or(String::new());
}

pub fn database_type() -> String {
    return env::var("DATABASE_TYPE").unwrap_or(String::new());
}

pub fn database_url() -> String {
    return env::var("DATABASE_URL").unwrap_or(String::new());
}

pub fn port() -> String {
    return env::var("PORT").unwrap_or(String::from("5000"));
}