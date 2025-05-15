use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct User{
    pub id: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Deserialize)]
pub struct SignupInput {
    pub email: String,
    pub password: String,
}