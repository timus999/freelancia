use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct BidRequest {
    #[validate(range(min = 1, message = "Job ID must be positive"))]
    pub job_id: u64,

    #[validate(length(min = 1, message = "Timeline is required"))]
    pub timeline: String,
    
    #[validate(range(min = 1, message = "Budget must be positive"))]
    pub budget: u64,

    #[validate(length(min = 1, max = 1000, message = "Message must be between 1 and 1000 characters"))]
    pub message: String,
}
