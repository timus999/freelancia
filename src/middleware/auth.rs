use axum::{
    http::{Request},
    middleware::Next,
    response::{Response},
    extract::State,
    Extension,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::error::AppError;
use crate::models::{auth::AuthUser, jwt::Claims};

pub async fn auth_middleware(
    State(pool): axum::extract::State<SqlitePool>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Extract the Authorization header and remove "Bearer " prefix
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    // Validate the presence of a valid Bearer token
    let token = auth_header.ok_or(AppError::Unauthorized(
        // Edge case: Missing or malformed Authorization header
        "Missing or invalid Authorization header".to_string(),
    ))?;

    // Decode the JWT using the secret key from environment variable
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").expect("JWT_SECRET must be set").as_ref()),
        &Validation::default(),
    )
    .map_err(|_| {
        // Edge case: Invalid token (e.g., malformed, expired, or incorrect signature)
        AppError::Unauthorized("Invalid token".to_string())
    })?
    .claims;

    // Fetch user data from database using the user_id from JWT claims
    let user = sqlx::query!(
        "SELECT wallet_address, verified_wallet FROM users WHERE id = ?",
        claims.user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        // Edge case: Database query failure (e.g., connection issues)
        AppError::Database(e.to_string())
    })?
    .ok_or(AppError::Unauthorized(
        // Edge case: User not found in database
        "User not found".to_string()
    ))?;

    // Create AuthUser struct to store authenticated user data
    let auth_user = AuthUser {
        id: claims.user_id,
        wallet_address: user.wallet_address,
        role: claims.role,
        verified_wallet: user.verified_wallet,
    };

    // Insert AuthUser into request extensions for downstream handlers
    let mut req = req;
    req.extensions_mut().insert(Arc::new(auth_user));

    // Proceed to the next middleware or handler
    Ok(next.run(req).await)
}

pub async fn freelancer_only(
    Extension(auth_user): Extension<Arc<AuthUser>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Restrict access to users with the "freelancer" role
    if auth_user.role != "freelancer" {
        // Edge case: User attempts access with a non-freelancer role
        return Err(AppError::Unauthorized("Freelancer role required".to_string()));
    }

    // Proceed to the next middleware or handler
    Ok(next.run(req).await)
}

pub async fn client_only(
    Extension(auth_user): Extension<Arc<AuthUser>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Restrict access to users with the "client" role
    if auth_user.role != "client" {
        // Edge case: User attempts access with a non-client role
        return Err(AppError::Unauthorized("Client role required".to_string()));
    }

    // Proceed to the next middleware or handler
    Ok(next.run(req).await)
}

pub async fn wallet_verified_only(
    State(pool): axum::extract::State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Fetch verified_wallet status from database
    let user = sqlx::query!(
        "SELECT verified_wallet FROM users WHERE id = ?",
        auth_user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        // Edge case: Database query failure (e.g., connection issues)
        AppError::Database(e.to_string())
    })?;

    // Require wallet verification if wallet_address is present
    if auth_user.wallet_address.is_some() && !user.verified_wallet {
        // Edge case: User has a wallet_address but it is not verified
        return Err(AppError::Unauthorized("Wallet verification required".to_string()));
    }

    // Proceed to the next middleware or handler
    Ok(next.run(req).await)
}