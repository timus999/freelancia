use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;

// use crate::handlers::freelancer::submit_bid;
use crate::handlers::{
    freelancer::*,
    job::apply_for_job,
    proposal::{create_proposal, get_my_proposals},
};
use crate::middleware::auth::{auth_middleware, freelancer_only};

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
        .route("/jobs/apply", post(apply_for_job))
        .route("/jobs/:job_id/status", get(get_job_user_status))
        .route("/my_jobs/:application_id", get(get_user_job_by_id))
        .route("/my_jobs/submit-deliverable", post(submit_job_deliverable))
        .route("/claim-timeout", post(claim_timeout))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            freelancer_only,
        ))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ))
        .with_state(pool)
}
