use serde::{Serialize, Deserialize};

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
}

#[derive(Serialize, Deserialize)]
pub struct ProfileResponse {
    pub email: String,
    pub wallet_address: String,
    pub role: String,
}

pub struct AuthUser {
    pub id: i64,
    pub wallet_address: String,
    pub role: String,
}

