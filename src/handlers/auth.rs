use axum::{Json, extract::State, http::StatusCode, response::{IntoResponse}, Extension};
use serde_json::json;
use sqlx::SqlitePool;
use validator::{Validate};
use crate::models::auth::*;
use crate::models::user::{SignupRequest};
use crate::utils::*;
use crate::error::AppError;
use std::sync::Arc;


pub async fn signup(
    State(pool): State<SqlitePool>,
    Json(payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::Validation)?;

    let password = payload.password.as_ref().ok_or_else(|| {
        AppError::Validation({
            let mut errors = validator::ValidationErrors::new();
            errors.add("password", validator::ValidationError::new("Password is required"));
            errors
        })
    })?;

    let hashed_password = hash_password(password)
        .map_err(|_| AppError::Server("Failed to hash password".to_string()))?;

    let result = sqlx::query!(
        "INSERT INTO users (email, password, wallet_address, role) VALUES (?, ?, ?, ?)",
        payload.email,
        hashed_password,
        payload.wallet_address,
        "freelancer" // Default role, adjust as needed
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "User signed up", "user_id": result.last_insert_rowid() })),
    ))
}


//login logic
pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {

    let password = payload.password.as_ref().ok_or(AppError::Validation({
        let mut errors = validator::ValidationErrors::new();
        errors.add("password", validator::ValidationError::new("Password is required"));
        errors
    }))?;

    let user = sqlx::query!(
        r#"
        SELECT id AS "id!: i64", password, role
        FROM users
        WHERE email = ?
        "#,
        payload.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::Unauthorized("Invalid credentials".to_string()))?;

    let user_password = user.password.as_ref().ok_or(AppError::Server(
        "User password not found".to_string(),
    ))?;

    if !verify_password(password, user_password)
        .map_err(|_| AppError::Server("Password verification failed".to_string()))?
    {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let token = generate_jwt(user.id, user.role)
        .map_err(|_| AppError::Server("Token generation failed".to_string()))?;

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            message: "Logged in".to_string(),
            token,
        }),
    ))
}

pub async fn profile(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
) -> Result<impl IntoResponse, AppError> {
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
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(ProfileResponse {
            email: user.email,
            wallet_address: auth_user.wallet_address.clone(),
            role: auth_user.role.clone(),
        }),
    ))
}