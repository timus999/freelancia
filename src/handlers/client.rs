use crate::error::AppError;
use crate::models::auth::AuthUser;
use crate::models::client::*;
use axum::{
    extract::{Extension, Json, Path, State},
    response::IntoResponse,
};
use sqlx::SqlitePool;
use std::sync::Arc;

pub async fn get_user_job_by_id(
    State(pool): State<SqlitePool>,
    Path(job_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT
            j.id,
            j.title,
            j.description,
            j.skills,
            j.budget,
            j.location,
            j.job_type,
            j.job_ipfs_hash,
            j.posted_at,
            j.deadline,
            j.client_id,
            j.category,
            j.status,
            d.submitted,
            d.submitted_at,
            d.disputed,
            d.disputed_at,
            d.review_requested,
            d.review_requested_at,
            d.cancelled,
            d.cancelled_at
        FROM jobs j
        LEFT JOIN job_applications ja ON ja.job_id = j.id
        LEFT JOIN job_deliverables d ON d.application_id = ja.id
        WHERE j.id = ?  
        "#,
        job_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(JobResponse {
        id: row.id,
        title: row.title,
        description: row.description,
        skills: row.skills,
        budget: row.budget,
        location: row.location,
        job_type: row.job_type,
        job_ipfs_hash: row.job_ipfs_hash,
        posted_at: row.posted_at,
        deadline: row.deadline,
        client_id: row.client_id,
        category: row.category,
        status: row.status,
        submitted: row.submitted,
        submitted_at: row.submitted_at,
        disputed: row.disputed,
        disputed_at: row.disputed_at,
        review_requested: row.review_requested,
        review_requested_at: row.review_requested_at,
        cancelled: row.cancelled,
        cancelled_at: row.cancelled_at,
    }))
}

pub async fn get_user_approved_job(
    State(pool): State<SqlitePool>,
    Path(job_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT ja.job_id,j.status, ja.approved_at, ja.applied_at, ja.freelancer_wallet, ja.id, jd.ipfs_hash, jd.submitted_at, jd.disputed, jd.disputed_at, jd.submitted, jd.review_requested,
	jd.review_requested_at, p.username
	FROM job_applications ja
	LEFT JOIN job_deliverables jd ON jd.application_id = ja.id
	LEFT JOIN profiles p ON p.user_id = ja.user_id
    LEFT JOIN jobs j ON j.id = ja.job_id
    LEFT JOIN users u ON u.id = j.client_id
	WHERE ja.job_id = ? AND ja.approved = 1;  
        "#,
        job_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(ApprovedWorkResponse {
        job_id: row.job_id,
        approved_at: row.approved_at,
        applied_at: row.applied_at,
        freelancer_wallet: row.freelancer_wallet,
        application_id: row.id,
        work_ipfs_hash: row.ipfs_hash,
        submitted_at: row.submitted_at,
        disputed: row.disputed,
        disputed_at: row.disputed_at,
        submitted: row.submitted,
        review_requested: row.review_requested,
        review_requested_at: row.review_requested_at,
        freelancer_username: row.username,
        job_status: row.status,
    }))
}
pub async fn review_request(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Path(application_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    // Ensure the current user is the client who owns this job
    let is_owner = sqlx::query!(
        r#"
        SELECT 
             j.client_id, p.username, ja.user_id as freelancer_id, j.id as job_id
            FROM job_deliverables d
            JOIN job_applications ja ON d.application_id = ja.id
            JOIN jobs j ON ja.job_id = j.id
            LEFT JOIN profiles p on p.user_id = j.client_id
            WHERE d.application_id = ? AND j.client_id = ?
        "#,
        application_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if is_owner.client_id != auth_user.id {
        return Err(AppError::Unauthorized(
            "You are not authorized to request a review for this deliverable.".into(),
        ));
    }

    let result = sqlx::query!(
        r#"
        UPDATE job_deliverables
        SET review_requested = 1,
            submitted = 0,
            review_requested_at = CURRENT_TIMESTAMP
        WHERE application_id = ?
        "#,
        application_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = "open"
        WHERE id = ?
        "#,
        is_owner.job_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Deliverable not found.".into()));
    }

    let freelancer_msg = format!(
        "{} has requested to review the work.",
        is_owner.username.unwrap_or_default()
    );

    // Step 3: Create notification for freelancer
    sqlx::query!(
        "INSERT INTO notifications (user_id, message, read, type, job_id, actor_id) VALUES (?, ?, 0, ?, ?, ?)",
        is_owner.freelancer_id,
        freelancer_msg,
        "resubmit",
        is_owner.job_id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let client_msg = format!("You have requested to review the work.");

    // Step 3: Create notification for client
    sqlx::query!(
        "INSERT INTO notifications (user_id, message, read, type, job_id, actor_id) VALUES (?, ?, 0, ?, ?, ?)",
        auth_user.id,
        client_msg,
        "work_revision",
        is_owner.job_id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json({
        serde_json::json!({
            "message": "Review updated successfully.",
        })
    }))
}

pub async fn approve_job_deliverable(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<ApproveDeliverablePayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Step 1: Fetch job application + job + profile
    let record = sqlx::query!(
        r#"
        SELECT ja.id AS application_id, ja.job_id, ja.user_id AS freelancer_id,
               j.client_id, j.title, p.username
        FROM job_applications ja
        JOIN jobs j ON ja.job_id = j.id
        LEFT JOIN profiles p ON p.user_id = ja.user_id
        WHERE ja.id = ?
        "#,
        payload.application_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let record = match record {
        Some(r) => r,
        None => return Err(AppError::NotFound("Application not found.".into())),
    };

    if record.client_id != auth_user.id {
        return Err(AppError::Unauthorized(
            "You are not the owner of this job.".into(),
        ));
    }

    // Step 2: Check deliverable is submitted
    let deliverable = sqlx::query!(
        "SELECT submitted FROM job_deliverables WHERE application_id = ?",
        payload.application_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let Some(d) = deliverable else {
        return Err(AppError::BadRequest(
            "No deliverable found for this application.".into(),
        ));
    };

    if !d.submitted {
        return Err(AppError::BadRequest(
            "Deliverable has not been submitted yet.".into(),
        ));
    }

    // Step 3: Approve work → update job + clear review_requested
    sqlx::query!(
        r#"
        UPDATE jobs SET status = 'completed' WHERE id = ?
        "#,
        record.job_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query!(
        r#"
        UPDATE job_deliverables
        SET review_requested = 0,
            review_requested_at = NULL
        WHERE application_id = ?
        "#,
        payload.application_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Step 4: Notify freelancer
    let freelancer_message = format!(
        "Your work for the job '{}' has been approved!",
        record.title
    );

    sqlx::query!(
        r#"
        INSERT INTO notifications (user_id, message, read, type, job_id, actor_id)
        VALUES (?, ?, 0, ?, ?, ?)
        "#,
        record.freelancer_id,
        freelancer_message,
        "completed",
        record.job_id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let client_message = format!("Your job has been sucessfully completed!!!");

    sqlx::query!(
        r#"
        INSERT INTO notifications (user_id, message, read, type, job_id, actor_id)
        VALUES (?, ?, 0, ?, ?, ?)
        "#,
        auth_user.id,
        client_message,
        "completed",
        record.job_id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Deliverable approved successfully"
    })))
}

pub async fn cancel_escrow(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<CancelEscrowPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Step 1: Fetch job info and validate ownership
    let job = sqlx::query!(
        r#"
        SELECT j.id, j.client_id, j.status, jd.cancelled
        FROM jobs j
        LEFT JOIN job_applications ja ON ja.job_id = j.id
        LEFT JOIN job_deliverables jd ON ja.id = jd.application_id
        WHERE j.id = ?
        "#,
        payload.job_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let job = match job {
        Some(j) => j,
        None => return Err(AppError::NotFound("Job not found".into())),
    };

    if job.client_id != auth_user.id {
        return Err(AppError::Unauthorized(
            "You are not the owner of this job.".into(),
        ));
    }

    // Step 2: Ensure job is not already submitted or cancelled
    if job.status == "submitted" {
        return Err(AppError::BadRequest(
            "Work has already been submitted.".into(),
        ));
    }

    if job.cancelled.unwrap_or(false) {
        return Err(AppError::BadRequest("Job is already cancelled.".into()));
    }

    // Step 3: Update status to 'cancelled'
    sqlx::query!(
        r#"
        UPDATE jobs SET status = 'rejected'
        WHERE id = ?
        "#,
        job.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Step 4: Notify freelancers who applied
    let applicants = sqlx::query!(
        r#"
        SELECT user_id FROM job_applications
        WHERE job_id = ? AND approved = 1
        "#,
        job.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    for applicant in applicants {
        sqlx::query!(
            r#"
            INSERT INTO notifications (user_id, message, read, type, job_id, actor_id)
            VALUES (?, ?, 0, 'cancelled', ?, ?)
            "#,
            applicant.user_id,
            "The job has been cancelled by the client.",
            job.id,
            auth_user.id
        )
        .execute(&pool)
        .await
        .ok(); // Non-critical error
    }

    Ok(Json(serde_json::json!({
        "message": "Escrow cancelled successfully."
    })))
}
