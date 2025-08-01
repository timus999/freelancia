use crate::error::AppError;
use crate::models::auth::AuthUser;
use crate::models::freelancer::*;
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
    Extension,
};
use chrono::{NaiveDateTime, Utc};
use sqlx::SqlitePool;
use std::{sync::Arc, time::Duration};

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
pub async fn get_user_job_by_id(
    State(pool): State<SqlitePool>,
    Path(application_id): Path<i64>,
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
            u.wallet_address,
            d.submitted,
            d.submitted_at,
            d.disputed,
            d.disputed_at,
            d.timeout_claimed,
            d.timeout_claimed_at
        FROM jobs j
        LEFT JOIN users u ON j.client_id = u.id
        LEFT JOIN job_applications ja ON ja.job_id = j.id
        LEFT JOIN job_deliverables d ON d.application_id = ja.id
        WHERE ja.id = ?  
        "#,
        application_id
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
        wallet_address: row.wallet_address,
        submitted: row.submitted,
        submitted_at: row.submitted_at,
        disputed: row.disputed,
        disputed_at: row.disputed_at,
        timeout_claimed: row.timeout_claimed,
        timeout_claimed_at: row.timeout_claimed_at,
    }))
}

pub async fn submit_job_deliverable(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<SubmitDeliverablePayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Step 1: Check that the application exists, belongs to the user, and is approved
    let application = sqlx::query!(
        r#"
        SELECT ja.id, ja.approved, ja.user_id, ja.job_id, j.title , j.client_id, p.username
        FROM job_applications ja
        LEFT JOIN jobs j ON j.id = ja.job_id
        LEFT JOIN profiles p ON ja.user_id = p.user_id
        WHERE ja.id = ?
        "#,
        payload.application_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let application = match application {
        Some(app) => app,
        None => return Err(AppError::NotFound("Application not found".into())),
    };

    if application.user_id != auth_user.id {
        return Err(AppError::Unauthorized(
            "You are not the owner of this application".into(),
        ));
    }

    if !application.approved.unwrap_or(false) {
        return Err(AppError::BadRequest(
            "Application has not been approved yet".into(),
        ));
    }
    let deliverable = sqlx::query!(
        r#"
    SELECT id, submitted, review_requested
    FROM job_deliverables
    WHERE application_id = ?
    "#,
        payload.application_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    //

    if let Some(existing) = deliverable {
        if existing.review_requested.unwrap_or(false) {
            // Update existing deliverable: reset review_requested and set new IPFS hash
            sqlx::query!(
            r#"
            UPDATE job_deliverables
            SET ipfs_hash = ?, submitted = 1, submitted_at = CURRENT_TIMESTAMP, review_requested = 0, review_requested_at = NULL
            WHERE application_id = ?
            "#,
            payload.ipfs_hash,
            payload.application_id
        )
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

            // Reset job status to "submitted"
            sqlx::query!(
                r#"UPDATE jobs SET status = 'submitted' WHERE id = ?"#,
                application.job_id
            )
            .execute(&pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        } else {
            return Err(AppError::BadRequest(
                "Work has already been submitted.".into(),
            ));
        }
    } else {
        // No existing record → insert new deliverable
        sqlx::query!(
            r#"
        INSERT INTO job_deliverables (application_id, ipfs_hash, submitted)
        VALUES (?, ?, 1)
        "#,
            payload.application_id,
            payload.ipfs_hash,
        )
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query!(
            r#"UPDATE jobs SET status = 'submitted' WHERE id = ?"#,
            application.job_id
        )
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    // msg for client
    let msg_client = format!(
        " {} has submitted work for your job '{}'.",
        application.username.unwrap_or_default(),
        application.title.clone().unwrap_or_default()
    );

    // Step 3: Create notification for client
    sqlx::query!(
        "INSERT INTO notifications (user_id, message, read, type, job_id, actor_id) VALUES (?, ?, 0, ?, ?, ?)",
        application.client_id,
        msg_client,
        "review",
        application.job_id,
        auth_user.id,

    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    let msg_freelancer = format!(
        "You have successfully submitted to the  job '{}'.",
        application.title.unwrap_or_default()
    );

    sqlx::query!(
        "INSERT INTO notifications (user_id, message, read, type, job_id, actor_id) VALUES (?, ?, 0, ?, ?, ?)",
        auth_user.id,
        msg_freelancer,
        "submitted",
        application.job_id,
        auth_user.id,

    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Job deliverable submitted successfully"
    })))
}

pub async fn claim_timeout(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<ClaimTimeoutPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Step 1: Get deliverable and application info
    let data = sqlx::query!(
        r#"
        SELECT jd.submitted, jd.submitted_at, jd.review_requested, jd.disputed,
               j.id as job_id, j.status, j.client_id, j.title, p.username
        FROM job_deliverables jd
        JOIN job_applications ja ON ja.id = jd.application_id
        JOIN jobs j ON j.id = ja.job_id
        LEFT JOIN profiles p ON ja.user_id = p.user_id
        WHERE j.id = ? AND ja.user_id = ?
        "#,
        payload.job_id,
        auth_user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let data = match data {
        Some(row) => row,
        None => return Err(AppError::NotFound("No submitted job found.".into())),
    };

    // Step 2: Validate
    if !data.submitted {
        return Err(AppError::BadRequest("Work not submitted.".into()));
    }

    if data.disputed.unwrap_or(false) {
        return Err(AppError::BadRequest("Job is in dispute.".into()));
    }

    if data.review_requested.unwrap_or(false) {
        return Err(AppError::BadRequest(
            "Client has already requested a review.".into(),
        ));
    }

    if data.status == "completed" {
        return Err(AppError::BadRequest(
            "Job already marked as completed.".into(),
        ));
    }

    // Step 3: Check time since submission
    let submitted_at = data.submitted_at;

    let submitted_at = NaiveDateTime::parse_from_str(&submitted_at, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| AppError::BadRequest("Invalid submitted_at datetime format.".into()))?;
    let now = Utc::now().naive_utc();

    let diff = now.signed_duration_since(submitted_at);
    if diff < chrono::Duration::days(3) {
        return Err(AppError::BadRequest(
            "Claim timeout period (3 days) not yet passed.".into(),
        ));
    }

    // Step 4: Update job status to completed
    sqlx::query!(
        r#"
        UPDATE jobs SET status = 'completed'
        WHERE id = ?
        "#,
        data.job_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Step 5: Create notifications
    let freelancer_msg = format!(
        "You successfully claimed the job '{}' due to client inactivity.",
        data.title
    );

    let client_msg = format!(
        "{} has claimed the job '{}' due to no response within 3 days.",
        data.username.unwrap_or("Freelancer".to_string()),
        data.title
    );

    sqlx::query!(
        r#"INSERT INTO notifications (user_id, message, read, type, job_id, actor_id)
        VALUES (?, ?, 0, 'claimed', ?, ?)"#,
        auth_user.id,
        freelancer_msg,
        data.job_id,
        auth_user.id,
    )
    .execute(&pool)
    .await
    .ok();

    sqlx::query!(
        r#"INSERT INTO notifications (user_id, message, read, type, job_id, actor_id)
        VALUES (?, ?, 0, 'claimed', ?, ?)"#,
        data.client_id,
        client_msg,
        data.job_id,
        auth_user.id,
    )
    .execute(&pool)
    .await
    .ok();

    Ok(Json(serde_json::json!({
        "message": "You have successfully claimed the job as completed due to client inactivity."
    })))
}
