use axum::{Router, routing::{post, get}, middleware};
use crate::handlers::{auth::*};
use sqlx::SqlitePool;
use crate::middleware::auth::auth_middleware;

pub fn protected_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/profile", get(profile).route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware)))
        .with_state(pool)
}