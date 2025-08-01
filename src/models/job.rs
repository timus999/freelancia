use serde::{Deserialize, Serialize};
use sqlx;
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
    #[validate(length(min = 1, message = "Category is required"))]
    pub category: String,
    #[validate(length(min = 1, message = "status is required"))]
    pub status: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
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
}

#[derive(Serialize, Deserialize)]
pub struct JobsResponse {
    pub jobs: Vec<JobResponse>,
}

// Query parameters for filtering jobs
#[derive(Debug, Deserialize, Validate)]
pub struct JobFilterQuery {
    #[validate(range(min = 1, message = "Id must be non-negative"))]
    pub id: Option<u64>,
    #[validate(length(min = 1))]
    pub keyword: Option<String>, //Search in title/description
    #[validate(range(min = 0))]
    pub min_budget: Option<i32>, // Minimum budget
    #[validate(range(min = 0))]
    pub max_budget: Option<i32>, // maximum budget
    #[validate(length(min = 1))]
    pub skills: Option<String>, // filter by skills
    #[validate(length(min = 1))]
    pub location: Option<String>, //filter by location
    #[validate(length(min = 1))]
    pub job_type: Option<String>, //filter by job type
    #[validate(range(min = 1))]
    pub client_id: Option<i64>, //filter by client_id
    #[validate(length(min = 1))]
    pub category: Option<String>, // filter by category
    #[validate(length(min = 1))]
    pub deadline_start: Option<String>, // e.g., "2025-05-20"
    #[validate(length(min = 1))]
    pub deadline_end: Option<String>, // e.g., "2025-06-01"
    #[validate(length(min = 1))]
    pub posted_at_start: Option<String>, // e.g., "2025-05-01"
    #[validate(length(min = 1))]
    pub posted_at_end: Option<String>, // e.g., "2025-05-17"
    #[validate(length(min = 1))]
    pub status: Option<String>, // e.g., "open"
    #[validate(length(min = 1))]
    pub sort_by: Option<String>, // sort by e.g. "budget:asc"
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>, //pagination: max 100
    #[validate(range(min = 0))]
    pub offset: Option<i64>, // Pagination offset
}

#[derive(Serialize)]
pub struct Categories {
    pub categories: Vec<String>,
}

#[derive(Deserialize, Validate)]
pub struct ApplyJobPayload {
    #[validate(range(min = 1, message = "Id must be non-negative"))]
    pub job_id: i64,
}

#[derive(Deserialize, Validate)]
pub struct ApproveApplicationPayload {
    #[validate(range(min = 1, message = "Id must be non-negative"))]
    pub application_id: i64,
}

#[derive(Deserialize, Validate)]
pub struct CreateEscrowPayload {
    #[validate(range(min = 1, message = "Id must be non-negative"))]
    pub application_id: i64,
    #[validate(length(equal = 44, message = "pda must be 44 characters long"))]
    pub escrow_pda: String,
}

#[derive(serde::Serialize)]
pub struct ApplicantResponse {
    pub application_id: Option<i64>,
    pub freelancer_id: i64,
    pub freelancer_username: String,
    pub skills: Option<String>,
    pub profile_ipfs_hash: Option<String>,
    pub applied_at: String,
    pub approved: Option<bool>,
    pub approved_at: Option<String>,
    pub freelancer_wallet: String,
}
#[derive(serde::Serialize)]
pub struct MyJobsResponse {
    pub job_id: i64,
    pub title: String,
    pub status: String,
    pub description: String,
    pub skills: String,
    pub budget: u64,
    pub location: String,
    pub posted_at: String,
    pub deadline: String,
    pub client_id: i64,
    pub applied_at: String,
    pub approved: Option<bool>,
    pub is_saved: i32,
    pub application_id: i64,
}

#[derive(Debug, Serialize)]
pub struct Notification {
    pub id: i64,
    pub message: String,
    pub read: bool,
    pub created_at: String,
    pub job_id: Option<i64>,
    pub job_title: Option<String>,
    pub username: Option<String>,
    pub redirect_url: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct MarkReadPayload {
    #[validate(range(min = 1, message = "Id must be non-negative"))]
    pub id: i64, // Notification ID
}
