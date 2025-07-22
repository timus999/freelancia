use crate::error::AppError;
use crate::models::auth::*;
use crate::models::profile::*;
use axum::{
    extract::{Extension, Json, Path, Query},
    response::IntoResponse,
};
use serde_json::json;
use sqlx::{Sqlite, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;

pub async fn create_or_update_profile(
    Extension(pool): Extension<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<UserProfilePayload>,
) -> Result<impl IntoResponse, AppError> {
    // Check if username is already taken by another user
    let existing = sqlx::query_scalar!(
        "SELECT user_id FROM profiles WHERE username = ? AND user_id != ?",
        payload.username,
        auth_user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::Conflict("Username is already taken".into()));
    }

    // Insert or update profile
    sqlx::query!(
        r#"
        INSERT INTO profiles (
            user_id, username, role, bio, skills, certifications, work_history, profile_ipfs_hash
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(user_id) DO UPDATE SET
            username = excluded.username,
            role = excluded.role,
            bio = excluded.bio,
            skills = excluded.skills,
            certifications = excluded.certifications,
            work_history = excluded.work_history,
            profile_ipfs_hash = excluded.profile_ipfs_hash,
            updated_at = CURRENT_TIMESTAMP
        "#,
        auth_user.id,
        payload.username,
        auth_user.role,
        payload.bio,
        payload.skills,
        payload.certifications,
        payload.work_history,
        payload.profile_ipfs_hash
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(UserProfileResponse {
        message: "Profile saved successfully.".into(),
    }))
}

pub async fn get_profile_by_user_id(
    Path(user_id): Path<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ProfileResponseByIdOrUsername>, AppError> {
    let profile = sqlx::query_as::<Sqlite, ProfileResponseByIdOrUsername>(
        r#"
        SELECT user_id, username, role, bio, skills, certifications, work_history,
            profile_ipfs_hash, created_at, updated_at
        FROM profiles
        WHERE user_id = ?
        "#,
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    match profile {
        Some(profile_data) => Ok(Json(profile_data)),
        None => Err(AppError::NotFound("Profile not found".into())),
    }
}
pub async fn get_profile_by_username(
    Path(username): Path<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ProfileResponseByIdOrUsername>, AppError> {
    let profile = sqlx::query_as::<Sqlite, ProfileResponseByIdOrUsername>(
        r#"
        SELECT user_id, username, role, bio, skills, certifications, work_history,
            profile_ipfs_hash, created_at, updated_at
        FROM profiles
        WHERE username = ?
        "#,
    )
    .bind(username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    match profile {
        Some(profile_data) => Ok(Json(profile_data)),
        None => Err(AppError::NotFound("Profile not found".into())),
    }
}

pub async fn check_username_availability(
    Query(params): Query<HashMap<String, String>>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let username = match params.get("username") {
        Some(u) => u,
        None => return Err(AppError::Validation(validator::ValidationErrors::new())),
    };

    let exists = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM profiles WHERE username = ?)",
        username
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let available = exists.unwrap_or(0) == 0;

    Ok(Json(json!({ "available": available })))
}

