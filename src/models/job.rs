use serde::{Deserialize, Serialize};
use validator::Validate;


#[derive(Serialize, Deserialize, Validate)]
pub struct JobRequest {
    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,
    #[validate(length(min = 1, message = "Skills are required"))]
    pub skills: String, // Comma-separated
    #[validate(range(min = 0, message = "Budget must be non-negative"))]
    pub budget: i64,
    #[validate(length(min = 1, message = "Location is required"))]
    pub location: String,
    #[validate(length(min = 1, message = "Job type is required"))]
    pub job_type: String,
    #[validate(length(min = 1, message = "IPFS hash is required"))]
    pub job_ipfs_hash: String,
    #[validate(length(min = 1, message = "Deadline is required"))]
    pub deadline: String, // ISO 8601 format
}

#[derive(Serialize, Deserialize)]
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
    pub client_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct JobsResponse {
    pub jobs: Vec<JobResponse>,
}