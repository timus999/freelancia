use axum::{
    extract::{Json, State},
    http::StatusCode,
    Extension,
    response::IntoResponse,
};

use chrono::Utc;
use serde_json::json;
use sqlx::SqlitePool;
use validator::Validate;

use crate::error::AppError;
use crate::models::auth::AuthUser;
use crate::models::job::*;
use std::sync::Arc;


pub async fn create_job(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload) : Json<JobRequest>,
) -> Result<impl IntoResponse, AppError> {

    payload.validate().map_err(AppError::Validation)?;

    let client_id = auth_user.id.to_string();
    let posted_at = Utc::now().to_rfc3339();

    let result = sqlx::query!(
        r#"
        INSERT INTO jobs (
        title, description, skills, budget, location, job_type, job_ipfs_hash,
        posted_at, deadline, client_id
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#,
    payload.title,
    payload.description,
    payload.skills,
    payload.budget,
    payload.location,
    payload.job_type,
    payload.job_ipfs_hash,
    posted_at,
    payload.deadline,
    client_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let job_id = result.last_insert_rowid();

    Ok((
        StatusCode::CREATED,
        Json(json!({ "message" : "job created", "job_id": job_id})),
    ))
}

pub async fn view_jobs(
    State(pool): State<SqlitePool>,
    Extension(_auth_user) : Extension<Arc<AuthUser>>,
) -> Result<impl IntoResponse, AppError>{

    let jobs = sqlx::query!(
        r#"
        SELECT
            id AS "id!: i64",
            title,
            description,
            skills,
            budget,
            location,
            job_type,
            job_ipfs_hash,
            posted_at,
            deadline,
            client_id
        FROM jobs
        ORDER BY posted_at DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let jobs_response = JobsResponse {
        jobs: jobs
        .into_iter()
        .map(|job| JobResponse {
            id: job.id,
            title: job.title,
            description: job.description,
            skills: job.skills,
            budget: job.budget,
            location: job.location,
            job_type: job.job_type,
            job_ipfs_hash: job.job_ipfs_hash,
            posted_at: job.posted_at,
            deadline: job.deadline,
            client_id: job.client_id,
        })
        .collect(),
    };

    Ok((StatusCode::OK, Json(jobs_response)))
}
