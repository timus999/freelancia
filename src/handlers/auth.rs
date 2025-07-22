use crate::config::jwt_secret;
use crate::error::AppError;
use crate::models::{auth::*, jwt::*};
use crate::utils::*;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use validator::Validate;

pub async fn signup(
    State(pool): State<SqlitePool>,
    Json(payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate the payload structure and constraints
    payload.validate().map_err(AppError::Validation)?;

    // Validate role against allowed values
    let valid_roles = vec!["client", "freelancer"];
    if !valid_roles.contains(&payload.role.as_str()) {
        // Edge case: Invalid role provided
        return Err(AppError::Validation(validator::ValidationErrors::new()));
    }

    // Ensure wallet_user or client roles provide wallet_address and signature
    // if (payload.role == "wallet_user" || payload.role == "client") &&
    //     (payload.wallet_address.is_none() || payload.signature.is_none()) {
    //     // Edge case: Missing wallet_address or signature for wallet-based roles
    //     return Err(AppError::Validation(validator::ValidationErrors::new()));
    // }

    // Ensure email_user provides a password
    // if payload.email && payload.password.is_none() {
    //     return Err(Ap.pError::Validation(validator::ValidationErrors::new()));
    // }

    // Extract and hash password if provided
    // let password = payload.password.as_ref().ok_or_else(|| {
    //     // Edge case: Password field is None when expected
    //     AppError::Validation({
    //         let mut errors = validator::ValidationErrors::new();
    //         errors.add(
    //             "password",
    //             validator::ValidationError::new("Password is required"),
    //         );
    //         errors
    //     })
    // })?;

    let hashed_password = hash_password(&payload.password)
        .map_err(|_| AppError::Server("Failed to hash password".to_string()))?;

    // Insert user into database with verified_wallet set to false
    let result = sqlx::query!(
        "INSERT INTO users (email, password, role, verified_wallet) VALUES (?, ?, ?, ?)",
        payload.email,
        hashed_password,
        payload.role,
        false
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        // Edge case: Database constraint violation (e.g., duplicate email or wallet_address)
        AppError::Database(e.to_string())
    })?;

    let role = payload.role.clone();
    // Generate JWT for authenticated user
    let token = generate_jwt(result.last_insert_rowid(), payload.role)
        .map_err(|_| AppError::Server("Token generation failed".to_string()))?;

    // Return success response with user_id
    Ok((
        StatusCode::OK,
        Json(SignupResponse {
            message: "Logged in".to_string(),
            token,
            user_id: result.last_insert_rowid(),
            role: role,
            wallet_user: false,
            verified_wallet: false,
        }),
    ))
}

pub async fn wallet_signup(
    State(pool): State<SqlitePool>,
    Json(payload): Json<WalletSignupRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    payload.validate().map_err(AppError::Validation)?;

    // Check if wallet already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE wallet_address = ?",
        payload.wallet_address
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest(
            "Wallet already registered".to_string(),
        ));
    }

    // Use placeholder email

    // Insert new wallet-based user
    let result = sqlx::query!(
        "INSERT INTO users ( wallet_address, role,wallet_user ,verified_wallet) VALUES (?,?,?, ?)",
        payload.wallet_address,
        payload.role,
        true,
        false,
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let role = payload.role.clone();
    // Generate JWT
    let token = generate_jwt(result.last_insert_rowid(), payload.role)
        .map_err(|_| AppError::Server("Token generation failed".to_string()))?;

    // Respond with token
    Ok((
        StatusCode::OK,
        Json(WalletSignupResponse {
            message: "Wallet login successful".to_string(),
            token: token,
            user_id: result.last_insert_rowid(),
            role: role,
            wallet_user: true,
            verified_wallet: false,
        }),
    ))
}

pub async fn wallet_connect(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<WalletConnectRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    payload.validate().map_err(AppError::Validation)?;

    // Check if wallet already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE wallet_address = ?",
        payload.wallet_address
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest(
            "Wallet already registered".to_string(),
        ));
    }

    // Use placeholder email

    // Insert new wallet-based user
    sqlx::query!(
        "UPDATE users
     SET wallet_address = ?, wallet_user = ?
     WHERE id = ?",
        payload.wallet_address,
        true,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Respond with token
    Ok((
        StatusCode::OK,
        Json(WalletConnectResponse {
            message: "Wallet login successful".to_string(),
            wallet_user: true,
        }),
    ))
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Extract password from payload
    let password = payload.password.as_ref().ok_or(AppError::Validation({
        // Edge case: Password field is missing
        let mut errors = validator::ValidationErrors::new();
        errors.add(
            "password",
            validator::ValidationError::new("Password is required"),
        );
        errors
    }))?;

    // Fetch user by email
    let user = sqlx::query!(
        r#"
        SELECT id AS "id!: i64", password, role, wallet_user, verified_wallet 
        FROM users
        WHERE email = ?
        "#,
        payload.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::Unauthorized("Invalid credentials".to_string()))?;

    // Verify password
    let user_password = user.password.as_ref().ok_or(AppError::Server(
        // Edge case: User exists but password is null in database
        "User password not found".to_string(),
    ))?;

    if !verify_password(password, user_password)
        .map_err(|_| AppError::Server("Password verification failed".to_string()))?
    {
        // Edge case: Password does not match
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let role = user.role.clone();
    let verified_wallet = user.verified_wallet;
    let wallet_user = user.wallet_user;
    // Generate JWT for authenticated user
    let token = generate_jwt(user.id, user.role)
        .map_err(|_| AppError::Server("Token generation failed".to_string()))?;

    // Return success response with token
    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            message: "Logged in".to_string(),
            token,
            user_id: user.id,
            role,
            wallet_user,
            verified_wallet,
        }),
    ))
}

pub async fn wallet_login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<WalletLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    payload.validate().map_err(AppError::Validation)?;

    // Look up user by wallet address
    let user = sqlx::query!(
        "SELECT id AS 'id!: i64', role, wallet_user, verified_wallet FROM users WHERE wallet_address = ?",
        payload.wallet_address
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::Unauthorized("Wallet not registered".to_string()))?;

    let role = user.role.clone();
    let verified_wallet = user.verified_wallet;
    let wallet_user = user.wallet_user;

    // Generate JWT
    let token = generate_jwt(user.id, user.role)
        .map_err(|_| AppError::Server("Token generation failed".to_string()))?;

    // Respond with token
    Ok((
        StatusCode::OK,
        Json(WalletLoginResponse {
            message: "Wallet login successful".to_string(),
            token,
            user_id: user.id,
            role,
            wallet_user,
            verified_wallet,
        }),
    ))
}

pub async fn logout(
    State(pool): State<SqlitePool>,
    Extension(_auth_user): Extension<Arc<AuthUser>>,
    Extension(token): Extension<String>, //raw jwt token from middleware
) -> Result<impl IntoResponse, AppError> {
    //Decode token to extract expiration time
    let secret = jwt_secret();
    let token_data: TokenData<Claims> = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|e| AppError::Unauthorized(e.to_string()))?;

    // Insert token into blacklisted_tokens
    // Note: Blacklisting is required because we cannot modify the token's exp
    // (which would create a new token) or force the client to stop using the original.
    // Storing in blacklisted_tokens ensures the token is rejected until its exp.
    sqlx::query!(
        "INSERT INTO blacklisted_tokens (token, expires_at) VALUES (?, ?)",
        token,
        token_data.claims.exp
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    //return success response
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Logged out successfully"})),
    ))
}

pub async fn profile_basic(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch user email by ID
    let user = sqlx::query!(
        r#"
        SELECT email
        FROM users
        WHERE id = ?
        "#,
        auth_user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        // Edge case: User not found (inconsistent auth_user data)
        AppError::Database(e.to_string())
    })?;

    // Return user profile
    Ok((
        StatusCode::OK,
        Json(ProfileResponse {
            email: user.email.expect("Not provided"),
            wallet_address: auth_user.wallet_address.clone(),
            role: auth_user.role.clone(),
            wallet_user: false,
            verified_wallet: auth_user.verified_wallet,
        }),
    ))
}

pub async fn profile_verified(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch user email by ID
    let user = sqlx::query!(
        r#"
        SELECT email
        FROM users
        WHERE id = ?
        "#,
        auth_user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        // Edge case: User not found (inconsistent auth_user data)
        AppError::Database(e.to_string())
    })?;

    // Restrict access to wallet_user or client roles
    if auth_user.role != "client" {
        // Edge case: User attempts access with unauthorized role
        return Err(AppError::Unauthorized(
            "Wallet user or client role required".to_string(),
        ));
    }

    // Return verified user profile
    Ok((
        StatusCode::OK,
        Json(ProfileResponse {
            email: user.email.expect("Not provided"),
            wallet_address: auth_user.wallet_address.clone(),
            role: auth_user.role.clone(),
            wallet_user: true,
            verified_wallet: auth_user.verified_wallet,
        }),
    ))
}

// Helper function to check if wallet is verified
async fn check_wallet_verified(pool: &SqlitePool, wallet_address: &str) -> Result<bool, AppError> {
    // Query verified_wallet status for wallet_address
    let user = sqlx::query!(
        "SELECT verified_wallet FROM users WHERE wallet_address = ?",
        wallet_address
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Return false if user not found, otherwise return verified_wallet status
    Ok(user.map_or(false, |u| u.verified_wallet))
}

pub async fn request_nonce(
    State(pool): State<SqlitePool>,
    Json(payload): Json<NonceRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate payload structure
    payload.validate().map_err(AppError::Validation)?;

    // Prevent nonce generation for verified wallets
    if check_wallet_verified(&pool, &payload.wallet_address).await? {
        // Edge case: Wallet already verified
        return Err(AppError::BadRequest("Wallet already verified".to_string()));
    }

    // Generate nonce and timestamps
    let nonce = generate_nonce();
    let created_at = Utc::now().to_rfc3339();
    let expires_at = Utc::now()
        .checked_add_signed(Duration::minutes(15))
        .expect("valid timestamp")
        .to_rfc3339();

    // Store nonce in database
    sqlx::query!(
        "INSERT OR REPLACE INTO nonces (wallet_address, nonce, created_at, expires_at) VALUES (?,?,?,?)",
        payload.wallet_address,
        nonce,
        created_at,
        expires_at
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        // Edge case: Database error (e.g., constraint violation)
        AppError::Database(e.to_string())
    })?;

    // Return nonce
    Ok((StatusCode::OK, Json(NonceResponse { nonce })))
}

pub async fn verify(
    State(pool): State<SqlitePool>,
    Json(payload): Json<VerifyRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate payload structure
    payload.validate().map_err(AppError::Validation)?;

    // Prevent verification for already verified wallets
    if check_wallet_verified(&pool, &payload.wallet_address).await? {
        // Edge case: Wallet already verified
        return Err(AppError::BadRequest("Wallet already verified".to_string()));
    }

    // Fetch and validate nonce
    let nonce_record = sqlx::query!(
        "SELECT nonce, expires_at FROM nonces WHERE wallet_address = ? AND nonce = ?",
        payload.wallet_address,
        payload.nonce
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::Unauthorized(
        "Invalid or expired nonce".to_string(),
    ))?;

    // Parse and check nonce expiration
    let expires_at =
        chrono::DateTime::parse_from_rfc3339(&nonce_record.expires_at).map_err(|_| {
            // Edge case: Invalid expires_at format in database
            AppError::Server("Invalid expiration time".to_string())
        })?;

    if Utc::now() > expires_at.with_timezone(&Utc) {
        // Edge case: Nonce has expired
        return Err(AppError::Unauthorized("Nonce Expired".to_string()));
    }

    // Verify signature

    let message = create_solana_sign_message(&payload.nonce, &payload.wallet_address);
    if !verify_solana_signature(&message, &payload.signature, &payload.wallet_address)? {
        return Err(AppError::Unauthorized("Invalid signature".to_string()));
    }

    // Fetch or create user
    let user = sqlx::query!(
        r#"
        SELECT id AS "id!: i64", role
        FROM users
        WHERE wallet_address = ?
        "#,
        payload.wallet_address
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let (user_id, role) = match user {
        Some(user) => {
            // Update existing user
            sqlx::query!(
                "UPDATE users SET verified_wallet = ? WHERE id = ?",
                true,
                user.id
            )
            .execute(&pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
            (user.id, user.role)
        }
        None => {
            // Auto-register user if not exists
            let email = format!("{}@freelancia.wallet", payload.wallet_address);
            let result = sqlx::query!(
                "INSERT INTO users (email, wallet_address, role, verified_wallet) VALUES (?,?,?,?)",
                email,
                payload.wallet_address,
                "wallet_user",
                true
            )
            .execute(&pool)
            .await
            .map_err(|e| {
                // Edge case: Database constraint violation (e.g., duplicate wallet_address)
                AppError::Database(e.to_string())
            })?;

            (result.last_insert_rowid(), "freelancer".to_string())
        }
    };

    // Delete used nonce
    sqlx::query!(
        "DELETE FROM nonces WHERE wallet_address = ? AND nonce = ?",
        payload.wallet_address,
        payload.nonce
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Generate JWT
    let token = generate_jwt(user_id, role)
        .map_err(|_| AppError::Server("Token generation failed".to_string()))?;

    // Return success response
    Ok((
        StatusCode::OK,
        Json(VerifyResponse {
            message: "Wallet verified".to_string(),
            token,
        }),
    ))
}

pub async fn cleanup_blacklisted_tokens(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query!("DELETE FROM blacklisted_tokens WHERE expires_at < ?", now)
        .execute(pool)
        .await?;
    Ok(())
}
