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
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = auth_header.ok_or(AppError::Unauthorized(
        "Missing or invalid Authorization header".to_string(),
    ))?;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").expect("JWT_SECRET must be set").as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?
    .claims;

    let user = sqlx::query!(
        "SELECT wallet_address FROM users WHERE id = ?",
        claims.user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::Unauthorized("User not found".to_string()))?;

    let auth_user = AuthUser {
        id: claims.user_id,
        wallet_address: user.wallet_address,
        role: claims.role,
    };

    let mut req = req;
    req.extensions_mut().insert(Arc::new(auth_user));
    Ok(next.run(req).await)
}

pub async fn freelancer_only(
    // State(pool): axum::extract::State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    if auth_user.role != "freelancer" {
        return Err(AppError::Unauthorized("Freelancer role required".to_string()));
    }

    Ok(next.run(req).await)
}

pub async fn client_only(
    // State(pool): axum::extract::State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    if auth_user.role != "client" {
        return Err(AppError::Unauthorized("Client role required".to_string()));
    }

    Ok(next.run(req).await)
}