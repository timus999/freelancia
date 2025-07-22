use crate::error::AppError;
use crate::models::jwt::Claims;
use bcrypt::{hash, verify, DEFAULT_COST};
use bs58;
use chrono::{Duration, Utc};
use ed25519_dalek::{VerifyingKey, Signature};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::env;
use std::convert::TryInto;


pub fn hash_password(password: &String) -> Result<String, bcrypt::BcryptError> {
    // Hash the provided password using bcrypt with default cost factor
    hash(password, DEFAULT_COST)
    // Edge case: Password too long or bcrypt internal error
}

pub fn verify_password(password: &String, hashed: &String) -> Result<bool, bcrypt::BcryptError> {
    // Verify if the provided password matches the hashed password
    verify(password, hashed)
    // Edge case: Invalid hash format or bcrypt internal error
}

pub fn generate_jwt(user_id: i64, role: String) -> Result<String, jsonwebtoken::errors::Error> {
    // Create JWT claims with user_id, role, and 24-hour expiration
    let claims = Claims {
        user_id,
        role,
        exp: (Utc::now() + Duration::hours(24)).timestamp() as i64,
    };

    // Encode JWT using the secret key from environment variable
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set")
                .as_ref(),
        ),
    )
    // Edge case: Missing JWT_SECRET env variable or encoding failure
}

pub fn generate_nonce() -> String {
    // Generate a 32-character random alphanumeric nonce
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
    // Edge case: Random number generator failure (highly unlikely)
}

pub fn create_solana_sign_message(nonce: &str, wallet_address: &str) -> String {
    format!(
        "Welcome to Freelancia!\n\nWallet: {}\nNonce: {}\n\nSign this message to authenticate.",
        wallet_address, nonce
    )
}

pub fn verify_solana_signature(
    message: &str,
    signature_base58: &str,
    wallet_address_base58: &str,
) -> Result<bool, AppError> {
  // Trim inputs to prevent whitespace issues
    let signature_base58 = signature_base58.trim();
    let wallet_address_base58 = wallet_address_base58.trim();

    // Decode public key
    let pubkey_bytes = bs58::decode(wallet_address_base58)
        .into_vec()
        .map_err(|e| AppError::Unauthorized(format!("Invalid wallet address: {}", e)))?;

    // Convert to fixed-size array
    let pubkey_arr: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| AppError::Unauthorized("Invalid public key length".into()))?;

    let public_key = VerifyingKey::from_bytes(&pubkey_arr)
        .map_err(|e| AppError::Unauthorized(format!("Invalid public key: {}", e)))?;

    // Decode signature
    let signature_bytes = bs58::decode(signature_base58)
        .into_vec()
        .map_err(|e| AppError::Unauthorized(format!("Invalid signature format: {}", e)))?;

    // Convert to fixed-size array
    let signature_arr: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| AppError::Unauthorized("Invalid signature length".into()))?;

    let signature = Signature::from_bytes(&signature_arr);

    // Reconstruct signed message
    let formatted_message = format!(
        "\x18Solana Signed Message:\n{}{}",
        message.chars().count(),  // Character count
        message
    );

    // Verify signature
    public_key
        .verify_strict(formatted_message.as_bytes(), &signature)
        .map(|_| true)
        .map_err(|e| AppError::Unauthorized(format!("Verification failed: {}", e)))
}

