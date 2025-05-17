use axum::{Router, routing::get};
use sqlx::SqlitePool;

use crate::handlers::{print_msg, check_health, hello};



pub mod common;
pub mod freelancer;
pub mod client;

pub fn create_routes(pool: SqlitePool) -> Router{
    Router::new()
        .route("/", get(print_msg))
        .route("/api/v1/ping", get(check_health))
        .route("/api/v1/hello", get(hello))
        .merge(client::router(pool.clone()))
        .merge(freelancer::router(pool))
}

pub fn auth_routes(pool:SqlitePool) -> Router{
    Router::new()
        .merge(common::protected_routes(pool))
}

