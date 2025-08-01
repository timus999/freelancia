use crate::error::AppError;
use crate::models::auth::AuthUser;
use crate::models::escrow::*;
use axum::{
    extract::{Extension, Json, Path, State},
    response::IntoResponse,
};
use serde_json::json;
use serde_json::Value;
use sqlx::SqlitePool;
use std::fs;
use std::sync::Arc;

pub async fn get_idl() -> Result<Json<Value>, String> {
    let path = "programs/escrow/target/idl/escrow.json";
    println!("Attempting to read IDL from: {}", path);

    fs::read_to_string(path)
        .map_err(|e| format!("File read error: {}", e))
        .and_then(|data| {
            println!("Successfully read IDL ({} bytes)", data.len());
            println!("Sample data: {}", &data[..data.len().min(200)]);

            serde_json::from_str(&data)
                .map(Json)
                .map_err(|e| format!("JSON parse error: {}", e))
        })
}

pub async fn get_escrow(
    State(pool): State<SqlitePool>,
    Path(escrow_pda): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT n.job_id, u.wallet_address 
	FROM notifications n 
	LEFT JOIN users u ON n.actor_id = u.id
	WHERE n.escrow_pda = ?; 
        "#,
        escrow_pda
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(EscrowResponse {
        job_id: row.job_id,
        wallet_address: row.wallet_address,
    }))
}

pub async fn raise_dispute(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<RaiseDisputePayload>,
) -> Result<impl IntoResponse, AppError> {
    // Step 1: Fetch job application + deliverable + client/freelancer IDs
    let result = sqlx::query!(
        r#"
        SELECT 
            ja.user_id AS freelancer_id,
            j.client_id,
            j.id AS job_id,
            j.title AS job_title,
            jd.id AS deliverable_id,
            jd.submitted,
            jd.disputed
        FROM job_applications ja
        JOIN jobs j ON j.id = ja.job_id
        JOIN job_deliverables jd ON jd.application_id = ja.id
        WHERE j.id = ?
        "#,
        payload.job_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let data = match result {
        Some(d) => d,
        None => {
            return Err(AppError::NotFound(
                "Application or deliverable not found".into(),
            ))
        }
    };

    // Step 2: Validate ownership
    let user_id = auth_user.id;
    let is_freelancer = user_id == data.freelancer_id;
    let is_client = user_id == data.client_id;

    if !is_freelancer && !is_client {
        return Err(AppError::Unauthorized(
            "You are not allowed to raise dispute for this job".into(),
        ));
    }

    // Step 3: Validate status
    if data.disputed.unwrap_or(false) {
        return Err(AppError::BadRequest("Dispute already raised".into()));
    }

    if !data.submitted {
        return Err(AppError::BadRequest("Deliverable not submitted yet".into()));
    }

    // Step 4: Find an admin (arbiter)
    let admin = sqlx::query!("SELECT id, wallet_address FROM users WHERE admin = 1 LIMIT 1")
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let (arbiter_id, arbiter_wallet) = match admin {
        Some(a) => (a.id, a.wallet_address),
        None => return Err(AppError::Unauthorized("Admin user not found".into())),
    };
    // Step 5: Update deliverable with dispute info
    sqlx::query!(
        r#"
        UPDATE job_deliverables
        SET disputed = 1, disputed_at = CURRENT_TIMESTAMP, arbiter_id = ?
        WHERE id = ?
        "#,
        arbiter_id,
        data.deliverable_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Step 6: Create notifications for client & freelancer
    let message = format!(
        "A dispute has been raised for the job '{}' and assigned to an arbiter.",
        data.job_title
    );

    // notify client
    sqlx::query!(
        r#"
        INSERT INTO notifications (user_id, message, read, type, job_id, actor_id)
        VALUES (?, ?, 0, 'dispute', ?, ?)
        "#,
        data.client_id,
        message,
        data.job_id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // notify freelancer
    sqlx::query!(
        r#"
        INSERT INTO notifications (user_id, message, read, type, job_id, actor_id)
        VALUES (?, ?, 0, 'dispute', ?, ?)
        "#,
        data.freelancer_id,
        message,
        data.job_id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(Json(json!({
        "message": "Dispute raised and arbiter assigned",
        "arbiter_id": arbiter_id,
        "arbiter_wallet": arbiter_wallet,
    })))
}

pub async fn get_disputed_jobs_for_arbiter(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
) -> Result<Json<Vec<DisputedJobDetail>>, AppError> {
    // Step 1: Only allow admin (arbiter)
    if !auth_user.admin.unwrap_or_default() {
        return Err(AppError::Unauthorized(
            "Only arbiters can access disputed jobs".into(),
        ));
    }

    // Step 2: Fetch all disputed jobs assigned to this arbiter
    let jobs = sqlx::query_as!(
        DisputedJobDetail,
        r#"
SELECT 
    j.id AS job_id,
    j.title AS title,
    j.description AS description,
    j.budget AS budget,
    j.skills AS skills,
    j.client_id,
    j.posted_at,
    j.deadline,
    j.job_ipfs_hash,
    jd.ipfs_hash as work_ipfs_hash,
    ja.user_id AS freelancer_id,
    ja.id as application_id,
    jd.submitted_at,
    j.status AS job_status,
    jd.arbiter_id,
    p.username AS client_username,
	pf.username AS freelancer_username,
    u.wallet_address AS client_wallet,
    ua.wallet_address AS freelancer_wallet
FROM job_deliverables jd
LEFT JOIN job_applications ja ON ja.id = jd.application_id
LEFT JOIN jobs j ON ja.job_id = j.id
LEFT JOIN profiles p ON j.client_id = p.user_id
LEFT JOIN profiles pf ON ja.user_id = pf.user_id
LEFT JOIN users u ON u.id = j.client_id
LEFT JOIN users ua ON ua.id = pf.user_id
WHERE jd.disputed = 1 AND jd.arbiter_id = ?
        "#,
        auth_user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(jobs))
}

pub async fn arbiter_resolve(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<ArbiterResolvePayload>,
) -> Result<Json<&'static str>, AppError> {
    // Step 1: Check if current user is an admin
    if !auth_user.admin.unwrap_or_default() {
        return Err(AppError::Unauthorized(
            "Only admin can resolve disputes".into(),
        ));
    }

    // Step 2: Check if job exists and is disputed and arbiter is assigned
    let deliverable = sqlx::query!(
        r#"
        SELECT  disputed, arbiter_id
        FROM job_deliverables j
        WHERE disputed = 1 AND application_id = ?
        "#,
        payload.application_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let Some(deliverable) = deliverable else {
        return Err(AppError::NotFound("Disputed job not found".into()));
    };

    if deliverable.arbiter_id != Some(auth_user.id) {
        return Err(AppError::Unauthorized(
            "You are not assigned arbiter for this job".into(),
        ));
    }

    // Step 3: Update resolution
    sqlx::query!(
        r#"
        UPDATE job_deliverables
        SET resolved = ?, disputed = 0
        WHERE application_id = ?
        "#,
        payload.resolved,
        payload.application_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = "completed"
        WHERE client_id = ?
        "#,
        payload.client_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Step 4: Notify both client and freelancer

    let resolution = if payload.resolved {
        "approved"
    } else {
        "rejected"
    };
    let message = format!(
        "Dispute for Job #{} was resolved and work was {}",
        payload.job_id, resolution
    );
    sqlx::query!(
        r#"
        INSERT INTO notifications (user_id, message, read, type, job_id, actor_id)
        VALUES (?, ?, 0, ?, ?, ?)
        "#,
        payload.client_id,
        message,
        "completed",
        payload.job_id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query!(
        r#"
        INSERT INTO notifications (user_id, message, read, type, job_id, actor_id)
        VALUES (?, ?, 0, ?, ?, ?)
        "#,
        payload.freelancer_id,
        message,
        "completed",
        payload.job_id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json("Dispute resolved successfully"))
}
