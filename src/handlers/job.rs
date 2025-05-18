use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    Extension,
    response::IntoResponse,
};
use chrono::Utc;
use serde_json::json;
use sqlx::{SqlitePool};
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
        posted_at, deadline, client_id, category, status
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        auth_user.id,
        payload.category,
        payload.status
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

//reducdant code but commenting for future references

// pub async fn view_jobs(
//     State(pool): State<SqlitePool>,
//     Extension(_auth_user): Extension<Arc<AuthUser>>,
// ) -> Result<impl IntoResponse, AppError> {
//     // Fetch all jobs from the jobs table, ordered by posted_at in descending order
//     let jobs = sqlx::query!(
//         r#"
//         SELECT
//             id AS "id!: i64",
//             title,
//             description,
//             skills,
//             budget,
//             location,
//             job_type,
//             job_ipfs_hash,
//             posted_at,
//             deadline,
//             client_id,
//             category
//         FROM jobs
//         ORDER BY posted_at DESC
//         "#
//     )
//     .fetch_all(&pool)
//     .await
//     .map_err(|e| {
//         // Edge case: Database query failure (e.g., connection issues, table not found)
//         AppError::Database(e.to_string())
//     })?;

//     // Map database records to JobResponse structs for the response
//     let jobs_response = JobsResponse {
//         jobs: jobs
//             .into_iter()
//             .map(|job| JobResponse {
//                 id: job.id,
//                 title: job.title,
//                 description: job.description,
//                 skills: job.skills,
//                 budget: job.budget,
//                 location: job.location,
//                 job_type: job.job_type,
//                 job_ipfs_hash: job.job_ipfs_hash,
//                 posted_at: job.posted_at,
//                 deadline: job.deadline,
//                 client_id: job.client_id,
//                 category: job.category,
//             })
//             .collect(),
//     };

//     // Return all jobs in the response
//     Ok((StatusCode::OK, Json(jobs_response)))
// }

pub async fn get_filtered_jobs(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Query(query): Query<JobFilterQuery>,
) -> Result<impl IntoResponse, AppError>{
    //validate query parameters
    query.validate().map_err(AppError::Validation)?;

    //build dynamic SQL query
    // Initialize SQL query to select job fields from jobs table
    let mut sql = String::from(
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
            j.status
        FROM jobs j
        "#,
    );
    let mut params: Vec<String> = Vec::new();

    //keyword search in title or description
    // if let Some(keyword) = &query.keyword {
    //     sql.push_str(" AND (title LIKE ? OR description LIKE ?)");
    //     let wildcard = format!("%{}%", keyword);
    //     params.push(wildcard.clone());
    //     params.push(wildcard);
    // }

    //handle keyword search with fts5
    if let Some(keyword) = &query.keyword {
        //join with jobs_fts table for full-text search
        sql.push_str(" INNER JOIN jobs_fts ON j.id = jobs_fts.job_id");
        //use MATCH for fts5 query; escape single quotes in keyword
        sql.push_str(" WHERE jobs_fts MATCH ?");
        let escaped_keyword = keyword.replace("'", "'");
        params.push(escaped_keyword);
    } else {
        sql.push_str(" WHERE 1=1");
    }
    //Min budget filter
    if let Some(min_budget) = query.min_budget {
        sql.push_str(" AND j.budget >= ?");
        params.push(min_budget.to_string());
    }

    // Max budget filter
    if let Some(max_budget) = query.max_budget {
        sql.push_str(" AND j.budget <= ?");
        params.push(max_budget.to_string());
    }

    //skills filter
    if let Some(skills) = &query.skills {
            sql.push_str(" AND j.skills LIKE ?");
            params.push(format!("%{}%", skills));
        }
    

    //location filter
    if let Some(job_type) = &query.job_type {
        sql.push_str(" AND j.job_type = ?");
        params.push(job_type.clone());
    }

    //client ID filter
    if let Some(client_id) = query.client_id {
        sql.push_str(" AND j.client_id = ?");
        params.push(client_id.to_string());
    }

    //Category filter
    if let Some(category) = &query.category {
        sql.push_str(" AND j.category = ?");
        params.push(category.clone());
    }

    //deadline start filter
    if let Some(deadline_start) = &query.deadline_start{
        sql.push_str(" AND j.deadline >= ?");
        params.push(deadline_start.clone());
    }

    //deadline end filter
    if let Some(deadline_end) = &query.deadline_end{
        sql.push_str(" AND j.deadline <= ?");
        params.push(deadline_end.clone());
    }

    //posted_at start filter
    if let Some(posted_at_start) = &query.posted_at_start{
        sql.push_str(" AND j.posted_at >= ?");
        params.push(posted_at_start.clone());
    }

    //posted_at end filter
    if let Some(posted_at_end) = &query.posted_at_end{
        sql.push_str(" AND j.posted_at <= ?");
        params.push(posted_at_end.clone());
    }

    //filter by status
    if let Some(status) = &query.status {
        sql.push_str(" AND j.status = ?");
        params.push(status.clone());
    }

    //role-based access: freelancers see all jobs, clients see only their jobs
    if auth_user.role == "client" {
        sql.push_str(" AND j.client_id =?");
        params.push(auth_user.id.to_string());
    }

    //sort by posted_at (latest first)
    // sql.push_str(" ORDER BY posted_at DESC");

    //Dynamic sorting
    if let Some(sort_by) = &query.sort_by {
        let parts: Vec<&str> = sort_by.split(':').collect();
        if parts.len() == 2{
            let column = match parts[0]{
                "budget" => "j.budget",
                "posted_at" => "j.posted_at",
                "deadline" => "j.deadline",
                _ => "j.posted_at", //default 
            };
            let direction = match parts[1].to_uppercase().as_str() {
                "ASC" => "ASC",
                "DESC" => "DESC",
                _ => "DESC", //default
            };
            sql.push_str(&format!(" ORDER BY {} {}", column, direction));
        } else {
            sql.push_str(" ORDER BY j.posted_at DESC");
        }
    } else {
        sql.push_str(" ORDER BY j.posted_at DESC");
    }

    //pagination
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    sql.push_str(" LIMIT ? OFFSET ?");
    params.push(limit.to_string());
    params.push(offset.to_string());

    // Execute dynamic query with individual bind calls
    let mut query = sqlx::query_as::<sqlx::Sqlite, JobResponse>(&sql);
    for param in params {
        query = query.bind(param);
    }
    let jobs = query
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            // Edge case: SQL syntax error or database connection failure
            AppError::Database(e.to_string())
        })?;

    //return filtered jobs
    let jobs_response = JobsResponse { jobs };
    Ok((StatusCode::OK, Json(jobs_response)))
}