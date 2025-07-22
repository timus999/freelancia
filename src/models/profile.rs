use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct UserProfilePayload {
    #[validate(length(min = 1, message = "username is required"))]
    pub username: String,

    #[validate(length(min = 1, message = "role is required"))]
    pub role: String, // Should be either "freelancer" or "client"

    #[validate(length(max = 255, message = "Bio should be less than 255 characters"))]
    pub bio: Option<String>,

    #[validate(length(min = 1, message = "skills is required"))]
    pub skills: Option<String>, // comma-separated

    pub certifications: Option<String>,
    pub work_history: Option<String>,

    #[validate(length(min = 1, message = "profile ipfs hash is required"))]
    pub profile_ipfs_hash: String,
}

#[derive(Serialize)]
pub struct UserProfileResponse {
    pub message: String,
}

#[derive(Serialize, FromRow)]
pub struct ProfileResponseByIdOrUsername {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub bio: Option<String>,
    pub skills: Option<String>,
    pub certifications: Option<String>,
    pub work_history: Option<String>,
    pub profile_ipfs_hash: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

