use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Type};
use validator::Validate;

//Proposal model for database operations
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Proposal {
    pub id: i64,
    pub job_id: i64,
    pub freelancer_id: i64,
    pub cover_letter: String,
    pub bid_amount: f64,
    pub status: ProposalStatus,
    pub created_at: i64, //Unix timestamp
}

//Enum for proposal status
#[derive(Debug, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "lowercase")] // added for query param deserialization
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum ProposalStatus {
    Submitted,
    Accepted,
    Rejected,
}

//DTO for creating a proposal
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProposal {
    #[validate(range(min = 1))]
    pub job_id: i64,
    #[validate(length(min = 10, max = 1000))]
    pub cover_letter: String,
    #[validate(range(min = 1.0))]
    pub bid_amount: f64,
}

//DTO for updating proposal status
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProposal {
    pub status: ProposalStatus // Must be accepted or rejected
}

//DTO for response
#[derive(Debug, Serialize, FromRow)]
pub struct ProposalResponse {
    pub id: i64,
    pub job_id: i64,
    pub freelancer_id: i64,
    pub cover_letter: String,
    pub bid_amount: f64,
    pub status: ProposalStatus,
    pub created_at: i64,
}

//Query params for filtering freelancer proposals 
#[derive(Debug, Deserialize)]
pub struct ProposalFilter{
    pub status: Option<ProposalStatus>, //filter by status
    #[serde(default = "default_sort_by")]
    pub sort_by: String, // Sort by status or job.title
}

fn default_sort_by() -> String{
    "status".to_string()
}