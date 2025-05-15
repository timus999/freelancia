use axum::{Router, routing::get, routing::post};

use crate::handlers::{print_msg, check_health, hello, get_jobs, get_freelancers, create_job, create_freelancer};
use crate::handlers::{auth::{signup, login}, profile::profile_handler};
use sqlx::SqlitePool;

pub fn create_routes() -> Router{
    Router::new()
        .route("/", get(print_msg))
        .route("/api/v1/ping", get(check_health))
        .route("/api/v1/hello", get(hello))
        .route("/api/v1/jobs", get(get_jobs).post(create_job))
        .route("/api/v1/freelancers", get(get_freelancers).post(create_freelancer))
        .route("/profile", get(profile_handler))
}

pub fn auth_routes(pool: SqlitePool) -> Router{
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .with_state(pool)
}