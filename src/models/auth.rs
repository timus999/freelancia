use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String, // Optional for Web3 users
    // #[validate(length(min = 1, message = "Wallet address is required"))]
    // pub wallet_address: Option<String>,
    // #[validate(length(min = 1, message = "Signature is required"))]
    // pub signature: Option<String>, // Required if no password
    // #[validate(length(min = 1, message = "Role is required"))]
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignupResponse {
    pub message: String,
    pub user_id: i64,
    pub token: String,
    pub role: String,
    pub wallet_user: bool,
    pub verified_wallet: bool,
}

#[derive(Validate, Deserialize)]
pub struct WalletConnectRequest {
    #[validate(length(equal = 44, message = "Invalid wallet address"))]
    pub wallet_address: String,
}

#[derive(Serialize, Deserialize)]
pub struct WalletConnectResponse {
    pub message: String,
    pub wallet_user: bool,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub wallet_address: Option<String>,
    pub signature: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub message: String,
    pub token: String,
    pub user_id: i64,
    pub role: String,
    pub wallet_user: bool,
    pub verified_wallet: bool,
}

#[derive(Deserialize, Validate)]
pub struct WalletLoginRequest {
    #[validate(length(equal = 44, message = "Invalid wallet address"))]
    pub wallet_address: String,
}

#[derive(Serialize)]
pub struct WalletLoginResponse {
    pub message: String,
    pub token: String,
    pub user_id: i64,
    pub role: String,
    pub wallet_user: bool,
    pub verified_wallet: bool,
}

#[derive(Deserialize, Validate)]
pub struct WalletSignupRequest {
    #[validate(length(equal = 44, message = "Invalid wallet address"))]
    pub wallet_address: String,

    #[validate(length(min = 1, message = "Role is required"))]
    pub role: String,
}

#[derive(Serialize)]
pub struct WalletSignupResponse {
    pub message: String,
    pub role: String,
    pub token: String,
    pub user_id: i64,
    pub wallet_user: bool,
    pub verified_wallet: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ProfileResponse {
    pub email: String,
    pub wallet_address: Option<String>,
    pub role: String,
    pub wallet_user: bool,
    pub verified_wallet: bool,
}

pub struct AuthUser {
    pub id: i64,
    pub wallet_address: Option<String>,
    pub role: String,
    pub verified_wallet: bool,
    pub admin: Option<bool>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct NonceRequest {
    #[validate(length(equal = 44, message = "Invalid wallet address"))]
    pub wallet_address: String,
}

#[derive(Serialize, Deserialize)]
pub struct NonceResponse {
    pub nonce: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct VerifyRequest {
    #[validate(length(equal = 44, message = "Invalid wallet address"))]
    pub wallet_address: String,
    #[validate(length(min = 1, message = "Signature is required"))]
    pub signature: String,
    #[validate(length(min = 1, message = "Nonce is required"))]
    pub nonce: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyResponse {
    pub message: String,
    pub token: String,
}
