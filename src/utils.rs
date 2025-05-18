use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Header, EncodingKey};
use std::env;
use ethers::types::{Address, Signature};
use ethers::utils::{to_checksum, hash_message};
use crate::models::jwt::Claims;
use crate::error::AppError;

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

pub fn create_eip712_message(nonce: &str, wallet_address: &str) -> String {
    // Format an EIP-712 compliant message for wallet authentication
    format!(
        "Welcome to Freelancia!\n\nPlease sign this message to authenticate your wallet.\n\nWallet: {}\nNonce: {}",
        wallet_address, nonce
    )
    // Edge case: None, as this is a simple string format
}

pub fn verify_eip712_signature(
    message: &str,
    signature: &str,
    wallet_address: &str,
) -> Result<bool, AppError> {
    // Decode the signature from hex, removing "0x" prefix
    let signature = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|e| {
            // Edge case: Invalid hex string or malformed signature
            AppError::Server(format!("Invalid Signature format: {}", e))
        })?;

    // Parse the signature bytes into an ethers Signature object
    let signature = Signature::try_from(signature.as_slice())
        .map_err(|e| {
            // Edge case: Signature bytes are invalid (e.g., wrong length)
            AppError::Server(format!("Invalid Signature: {}", e))
        })?;

    // Hash the message using EIP-191 (Ethereum signed message standard)
    let msg_hash = hash_message(message);

    // Recover the address that signed the hashed message
    let recovered = signature
        .recover(msg_hash)
        .map_err(|e| {
            // Edge case: Signature recovery failure (e.g., invalid v value)
            AppError::Server(format!("Signature recovery failed: {}", e))
        })?;

    // Parse the expected wallet address
    let expected_address: Address = wallet_address
        .parse()
        .map_err(|e| {
            // Edge case: Invalid wallet address format (e.g., not a valid hex address)
            AppError::Server(format!("Invalid wallet address: {}", e))
        })?;

    // Compare recovered and expected addresses (case-insensitive)
    let is_valid = to_checksum(&recovered, None) == to_checksum(&expected_address, None);
    if !is_valid {
        // Log verification failure for debugging
        eprintln!(
            "Verification failed: recovered = {}, expected = {}",
            to_checksum(&recovered, None),
            to_checksum(&expected_address, None)
        );
    }
    // Return true if addresses match, false otherwise
    Ok(recovered == expected_address)
}