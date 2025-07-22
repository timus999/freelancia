use crate::error::AppError;
use crate::models::auth::AuthUser;
use crate::models::freelancer::*;
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
    Extension,
};
use sqlx::SqlitePool;
use std::sync::Arc;

pub async fn get_job_user_status(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Path(job_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let status = sqlx::query_as!(
        JobInteractionStatus,
        r#"
        SELECT 
            COALESCE(applied, false) AS applied,
            COALESCE(saved, false) AS saved
        FROM job_user_interactions
        WHERE user_id = ? AND job_id = ?
        "#,
        auth_user.id,
        job_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .unwrap_or(JobInteractionStatus {
        applied: false,
        saved: false,
    });

    Ok(Json(status))
}

