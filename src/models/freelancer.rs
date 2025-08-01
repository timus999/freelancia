use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize)]
pub struct JobInteractionStatus {
    pub applied: bool,
    pub saved: bool,
}

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
    pub wallet_address: Option<String>,
    pub submitted: Option<bool>,
    pub submitted_at: Option<String>,
    pub disputed: Option<bool>,
    pub disputed_at: Option<String>,
    pub timeout_claimed: Option<bool>,
    pub timeout_claimed_at: Option<String>,
}

#[derive(Deserialize)]
pub struct SubmitDeliverablePayload {
    pub application_id: i64,
    pub ipfs_hash: String,
}

#[derive(serde::Deserialize)]
pub struct ClaimTimeoutPayload {
    pub job_id: i64,
}
