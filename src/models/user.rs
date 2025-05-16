use serde::{Serialize, Deserialize};
use validator::Validate;

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct User{
    pub id: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SignupInput {

    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}