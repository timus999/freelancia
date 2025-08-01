use serde::{Deserialize, Serialize};

use sqlx::FromRow;
#[derive(Serialize)]
pub struct EscrowResponse {
    pub job_id: Option<i64>,
    pub wallet_address: Option<String>,
}

#[derive(Deserialize)]
pub struct RaiseDisputePayload {
    pub job_id: i64,
}

#[derive(Serialize, FromRow)]
pub struct DisputedJobDetail {
    pub job_id: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub budget: Option<i64>,
    pub skills: Option<String>,
    pub client_id: Option<i64>,
    pub posted_at: Option<String>,
    pub deadline: Option<String>,
    pub job_ipfs_hash: Option<String>,
    pub work_ipfs_hash: Option<String>,
    pub client_username: Option<String>,
    pub freelancer_username: Option<String>,
    pub application_id: Option<i64>,
    pub freelancer_id: Option<i64>,
    pub submitted_at: Option<String>,
    pub job_status: Option<String>,
    pub arbiter_id: Option<i64>,
    pub freelancer_wallet: Option<String>,
    pub client_wallet: Option<String>,
}

#[derive(Deserialize)]
pub struct ArbiterResolvePayload {
    pub job_id: i64,
    pub resolved: bool,
    pub application_id: i64,
    pub client_id: i64,
    pub freelancer_id: i64,
}
