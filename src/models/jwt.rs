use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub role: String, // "freelancer" or "client"
    pub exp: i64,
}

//model for blacklisted tokens stored in the database
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlacklistedToken {
    pub token:String,
    pub expires_at: i64, //unix timestamp in seconds
}

