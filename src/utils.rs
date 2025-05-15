use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};
use password_hash::{SaltString};
use rand::rngs::OsRng;
use crate::models::jwt::Claims;
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::config;
use chrono::{Utc, Duration};

pub fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();
    Ok(hash)
}

//verifies a password against a hash using Argon2
pub fn verify_password(password: &str, hash: &str) -> Result<(), ()> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| ())?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| ())
}

//generates a jwt token for the given user ID
pub fn generate_jwt(user_id: &str) -> Result<String, ()> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims{
        sub: user_id.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config::jwt_secret().as_bytes()),
    )
    .map_err(|_| ())
}