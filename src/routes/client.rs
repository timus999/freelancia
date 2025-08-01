use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use sqlx::SqlitePool;

use crate::handlers::client::*;
use crate::handlers::job::*;
use crate::handlers::proposal::{get_proposals_by_job, update_proposal};
use crate::middleware::auth::{auth_middleware, client_only, wallet_verified_only};

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/jobs/create", post(create_job))
        .route("/proposals/job/:job_id", get(get_proposals_by_job))
        .route("/proposals/:id", patch(update_proposal))
        .route("/applications/approve", post(approve_application))
        .route("/jobs/:id/applicants", get(get_job_applicants))
        .route("/jobs/:job_id", get(get_user_job_by_id))
        .route("/my-jobs/:job_id", get(get_user_approved_job))
        .route("/jobs/create-escrow", post(create_escrow_notification))
        .route("/review-request/:application_id", post(review_request))
        .route("/approve-work", post(approve_job_deliverable))
        .route("/cancel-escrow", post(cancel_escrow))
        .route_layer(middleware::from_fn_with_state(pool.clone(), client_only))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            wallet_verified_only,
        ))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ))
        .with_state(pool)
}
