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
    Json(payload): Json<JobRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate the payload structure and constraints (e.g., required fields, string lengths)
    payload.validate().map_err(AppError::Validation)?;

    // Record the current timestamp for when the job is posted
    let posted_at = Utc::now().to_rfc3339();

    // Insert job into the jobs table with provided details and authenticated user's ID as client_id
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
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        // Edge case: Database constraint violation (e.g., invalid foreign key, null in non-nullable field)
        AppError::Database(e.to_string())
    })?;

    // Retrieve the ID of the newly inserted job
    let job_id = result.last_insert_rowid();

    // Return success response with job_id
    Ok((
        StatusCode::CREATED,
        Json(json!({ "message": "job created", "job_id": job_id })),
    ))
}

pub async fn view_jobs(
    State(pool): State<SqlitePool>,
    Extension(_auth_user): Extension<Arc<AuthUser>>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch all jobs from the jobs table, ordered by posted_at in descending order
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
    .map_err(|e| {
        // Edge case: Database query failure (e.g., connection issues, table not found)
        AppError::Database(e.to_string())
    })?;

    // Map database records to JobResponse structs for the response
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

    // Return all jobs in the response
    Ok((StatusCode::OK, Json(jobs_response)))
}