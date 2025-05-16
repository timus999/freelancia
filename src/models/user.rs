use serde::{Serialize, Deserialize};
use validator::Validate;


#[derive(Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: Option<String>, // Optional for Web3 users
    #[validate(length(min = 1, message = "Wallet address is required"))]
    pub wallet_address: String,
    #[validate(length(min = 1, message = "Signature is required"))]
    pub signature: Option<String>, // Required if no password
}