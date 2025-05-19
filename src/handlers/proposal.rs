use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    Json,
    Extension,
};
use sqlx::{SqlitePool};
use serde_json::json;
use validator::Validate;
use std::sync::Arc;
use crate::{
    error::AppError,
    models::{
        proposal::{CreateProposal, ProposalResponse, ProposalStatus, UpdateProposal, ProposalFilter},
        auth::AuthUser,
    },
};

//Create a new proposal
//only authenticated freelancers can submit proposals
pub async fn create_proposal(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<CreateProposal>,
) -> Result<impl axum::response::IntoResponse, AppError>{
    //Ensure user is a freelancer
    if auth_user.role != "freelancer" {
        return Err(AppError::Unauthorized("Only freelancers can submit proposals".to_string()));
    }

    //validate input (job_id > 0, cover_letter 10-1000 chars, bid_amount > 1.0)
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    //check if job exits
    let job_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM jobs WHERE id = ?")
                        .bind(payload.job_id)
                        .fetch_one(&pool)
                        .await
                        .map_err(|e| AppError::Database(e.to_string()))?;
    
    if job_exists == 0 {
        return Err(AppError::BadRequest("Job does not exist".to_string()));
    }

    //Check if freelancer already applied
    let already_applied = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM proposals WHERE job_id = ? AND freelancer_id = ?"
    )
    .bind(payload.job_id)
    .bind(auth_user.id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if already_applied > 0 {
        return Err(AppError::BadRequest("You have already applied for this job".to_string()));
    }

    //Insert proposal
    let now = chrono::Utc::now().timestamp();
    let proposal = sqlx::query_as::<_, ProposalResponse>(
        r#"
        INSERT INTO proposals (job_id, freelancer_id, cover_letter, bid_amount, status, created_at)
        VALUES (?,?,?,?,?,?)
        RETURNING id, job_id, freelancer_id, cover_letter, bid_amount, status, created_at
        "#
    )
    .bind(payload.job_id)
    .bind(auth_user.id)
    .bind(payload.cover_letter)
    .bind(payload.bid_amount)
    .bind(ProposalStatus::Submitted)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "proposal" : proposal})),
    ))
}

//get proposals for a job (only job owner)
pub async fn get_proposals_by_job(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Path(job_id): Path<i64>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    //check if the user is the job owner
    let is_owner = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM jobs WHERE id = ? AND client_id = ?"
    )
    .bind(job_id)
    .bind(auth_user.id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if is_owner == 0 {
        return Err(AppError::Unauthorized("You are not the job owner".to_string()));
    }

    //Fetch proposals
    let proposals = sqlx::query_as::<_, ProposalResponse>(
        "SELECT id, job_id, freelancer_id, cover_letter, bid_amount, status, created_at
        FROM proposals WHERE job_id = ?"
    )
    .bind(job_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(json!({ "proposals" : proposals })),
    ))
}


//Update proposal status (accept/reject), restricted to job owner
pub async fn update_proposal(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Path(proposal_id): Path<i64>,
    Json(payload): Json<UpdateProposal>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    //Restricts to clients
    if auth_user.role != "client" {
        return Err(AppError::Unauthorized("Only clients can update proposals".to_string()));
    }

    //Validate input (status must be Accepted or Rejected)
    payload.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    if payload.status == ProposalStatus::Submitted {
        return Err(AppError::BadRequest("Cannot set status to submitted".to_string()));
    }

    //Atomically update proposal if user is job owner and proposal is submitted 
    let result = sqlx::query_as::<_, ProposalResponse>(
        r#"
        UPDATE proposals
        SET status = ?
        WHERE id = ? AND job_id IN (
            SELECT id FROM jobs WHERE client_id = ?
        ) AND status = ?
         RETURNING id, job_id, freelancer_id, cover_letter, bid_amount, status, created_at
         "#
    )
    .bind(&payload.status)
    .bind(proposal_id)
    .bind(auth_user.id)
    .bind(ProposalStatus::Submitted)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let proposal = match result {
        Some(proposal) => proposal,
        None => return Err(AppError::BadRequest(
            "Proposal not found, not owned, or not in submitted status".to_string()
        )),
    };

    //Return updated proposal
    Ok((
        StatusCode::OK,
        Json(json!({ "proposal": proposal})),
    ))
}

// Get freelancer's own proposal with filtering and sorting
pub async fn get_my_proposals(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Query(filter): Query<ProposalFilter>,
) -> Result<impl axum::response::IntoResponse, AppError> {
       // Restrict to freelancers
       if auth_user.role != "freelancer" {
        return Err(AppError::Unauthorized("Only freelancers can view their proposals".to_string()));
    }

    // Build complete query string
    let mut query = String::from(
        "SELECT p.id, p.job_id, p.freelancer_id, p.cover_letter, p.bid_amount, p.status, p.created_at
         FROM proposals p
         JOIN jobs j ON p.job_id = j.id
         WHERE p.freelancer_id = ?"
    );

    // Add status filter if provided
    if filter.status.is_some() {
        query.push_str(" AND p.status = ?");
    }

    // Add sorting
    let sort_column = match filter.sort_by.as_str() {
        "job.title" => "j.title",
        _ => "p.status", // Default to status
    };
    query.push_str(&format!(" ORDER BY {} ASC", sort_column));

    // Create query builder with complete query
    let mut query_builder = sqlx::query_as::<_, ProposalResponse>(&query)
        .bind(auth_user.id);

    // Bind status if provided
    if let Some(status) = &filter.status {
        // Convert ProposalStatus to lowercase string for TEXT column
        let status_str = match status {
            ProposalStatus::Submitted => "submitted",
            ProposalStatus::Accepted => "accepted",
            ProposalStatus::Rejected => "rejected",
        };
        query_builder = query_builder.bind(status_str);
    }

    // Fetch proposals
    let proposals = query_builder
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Return proposals
    Ok((
        StatusCode::OK,
        Json(json!({ "proposals": proposals })),
    ))
}
