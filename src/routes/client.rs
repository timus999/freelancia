use axum::{Router, routing::{get, post, patch}, middleware};
use sqlx::SqlitePool;

use crate::handlers::proposal::{get_proposals_by_job, update_proposal};
use crate::middleware::auth::{client_only, auth_middleware, wallet_verified_only};
use crate::handlers::job::create_job;


pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/jobs", post(create_job))
        .route("/proposals/job/:job_id", get(get_proposals_by_job))
        .route("/proposals/:id", patch(update_proposal))
        .route_layer(middleware::from_fn_with_state(pool.clone(), client_only))
        .route_layer(middleware::from_fn_with_state(pool.clone(), wallet_verified_only))
        .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware))
        .with_state(pool)
}