use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Header, EncodingKey};
use std::env;

use crate::models::jwt::Claims;

pub fn hash_password(password: &String) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &String, hashed: &String) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hashed)
}

pub fn generate_jwt(user_id: i64, role: String) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        user_id,
        role,
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set")
                .as_ref(),
        ),
    )
}