use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct JobResponse {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub skills: String,
    pub budget: i64,
    pub location: String,
    pub job_type: String,
    pub job_ipfs_hash: String,
    pub posted_at: String,
    pub deadline: String,
    pub client_id: i64,
    pub category: String,
    pub status: String,
    pub submitted: Option<bool>,
    pub submitted_at: Option<String>,
    pub disputed: Option<bool>,
    pub disputed_at: Option<String>,
    pub review_requested: Option<bool>,
    pub review_requested_at: Option<String>,
    pub cancelled: Option<bool>,
    pub cancelled_at: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct ApprovedWorkResponse {
    pub job_id: i64,
    pub approved_at: Option<String>,
    pub applied_at: String,
    pub freelancer_wallet: String,
    pub application_id: i64,
    pub work_ipfs_hash: Option<String>,
    pub submitted_at: Option<String>,
    pub disputed: Option<bool>,
    pub disputed_at: Option<String>,
    pub submitted: Option<bool>,
    pub review_requested: Option<bool>,
    pub review_requested_at: Option<String>,
    pub freelancer_username: Option<String>,
    pub job_status: Option<String>,
}

#[derive(Deserialize)]
pub struct ApproveDeliverablePayload {
    pub application_id: i64,
}

#[derive(Deserialize)]
pub struct CancelEscrowPayload {
    pub job_id: i64,
}
