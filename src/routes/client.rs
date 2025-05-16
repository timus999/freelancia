use axum::{Router, routing::post, middleware};
use sqlx::SqlitePool;

use crate::middleware::auth::{client_only, auth_middleware};
use crate::handlers::job::create_job;


pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/jobs", post(create_job).route_layer(middleware::from_fn_with_state(pool.clone(), client_only)))
        .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware))
        .with_state(pool)
}