use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub role: String, // "freelancer" or "client"
    pub exp: usize,
}

