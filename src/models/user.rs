use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct User{
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
}

#[derive(Deserialize)]
pub struct SignupInput {
    pub email: String,
    pub password: String,
}