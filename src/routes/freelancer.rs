use axum::{Router, routing::{get, post}, middleware};
use sqlx::SqlitePool;

// use crate::handlers::freelancer::submit_bid;
use crate::middleware::auth::{freelancer_only, auth_middleware};
use crate::handlers::{proposal::{create_proposal, get_my_proposals}};

// pub fn router(pool: SqlitePool) -> Router {
//     Router::new()
//         .route("/jobs/:id/bid", post(submit_bid))
//         .with_state(pool)
// }

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        // .route("/jobs", get(view_jobs))
        .route("/proposals", post(create_proposal))
        .route("/proposals/me", get(get_my_proposals))
        .route_layer(middleware::from_fn_with_state(pool.clone(), freelancer_only))
        .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware))
        .with_state(pool)
}