use crate::error::AppError;
use crate::models::auth::AuthUser;
use crate::models::job::*;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use chrono::Utc;
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use validator::Validate;

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
) -> Result<impl IntoResponse, AppError> {
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

    if let Some(id) = query.id {
        // Fetch by ID directly
        sql.push_str(" WHERE j.id = ?");
        params.push(id.to_string());
    } else {
        // Keyword-based search with FTS5
        if let Some(keyword) = &query.keyword {
            sql.push_str(" INNER JOIN jobs_fts ON j.id = jobs_fts.job_id");
            sql.push_str(" WHERE jobs_fts MATCH ?");
            let escaped_keyword = keyword.replace("'", ""); // sanitize input
            params.push(escaped_keyword);
        } else {
            sql.push_str(" WHERE 1=1");
        }

        // Add other filters like budget, skills, etc.
        if let Some(category) = &query.category {
            sql.push_str(" AND j.category = ?");
            params.push(category.clone());
        }

        // Add more filters here as needed...
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
    if let Some(deadline_start) = &query.deadline_start {
        sql.push_str(" AND j.deadline >= ?");
        params.push(deadline_start.clone());
    }

    //deadline end filter
    if let Some(deadline_end) = &query.deadline_end {
        sql.push_str(" AND j.deadline <= ?");
        params.push(deadline_end.clone());
    }

    //posted_at start filter
    if let Some(posted_at_start) = &query.posted_at_start {
        sql.push_str(" AND j.posted_at >= ?");
        params.push(posted_at_start.clone());
    }

    //posted_at end filter
    if let Some(posted_at_end) = &query.posted_at_end {
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
        if parts.len() == 2 {
            let column = match parts[0] {
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
    let jobs = query.fetch_all(&pool).await.map_err(|e| {
        // Edge case: SQL syntax error or database connection failure
        AppError::Database(e.to_string())
    })?;

    //return filtered jobs
    let jobs_response = JobsResponse { jobs };
    Ok((StatusCode::OK, Json(jobs_response)))
}

pub async fn get_categories(
    State(_pool): State<SqlitePool>,
) -> Result<(StatusCode, Json<Categories>), AppError> {
    // Mock categories (replace with DB query if needed)
    let categories = vec![
        "Web Development",
        "Graphic Design",
        "Writing",
        "Digital Marketing",
        "Video Editing",
        "Music Production",
    ];

    let categories = Categories {
        categories: categories.into_iter().map(String::from).collect(),
    };

    Ok((StatusCode::OK, Json(categories)))
}

pub async fn apply_for_job(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<Arc<AuthUser>>,
    Json(payload): Json<ApplyJobPayload>,
) -> Result<impl IntoResponse, AppError> {
    // Check if job exists and is open
    let job_status: Option<String> =
        sqlx::query_scalar!("SELECT status FROM jobs WHERE id = ?", payload.job_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

    match job_status {
        None => return Err(AppError::NotFound("Job not found".into())),
        Some(status) if status != "open" => {
            return Err(AppError::BadRequest(
                "Job is not open for applications".into(),
            ))
        }
        _ => {}
    }

    // Fetch the job to get creator's ID
    let job_creator = sqlx::query!(
        "SELECT j.client_id, p.username, u.wallet_address
            FROM jobs j
            JOIN profiles p ON p.user_id = ?
            JOIN users u ON u.id = ?
            WHERE j.id = ?",
        auth_user.id,
        auth_user.id,
        payload.job_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Try inserting application
    let result = sqlx::query!(
        "INSERT INTO job_applications (user_id, job_id, freelancer_wallet) VALUES (?, ?, ?)",
        auth_user.id,
        payload.job_id,
        job_creator.wallet_address
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            // ✅ Only insert notification after successful application
            let message = format!("{} has applied to your job post.", job_creator.username);
            sqlx::query!(
                "INSERT INTO notifications (user_id, message, read,type,  job_id, actor_id) VALUES (?, ?, 0, ?, ?, ?)",
                job_creator.client_id,
                message,
                "applied",
                payload.job_id,
                auth_user.id
            )
            .execute(&pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            sqlx::query!(
                "
                INSERT INTO job_user_interactions (user_id, job_id, applied)
                VALUES (?, ?, TRUE)
                ON CONFLICT(user_id, job_id) DO UPDATE SET applied = TRUE",
                auth_user.id,
                payload.job_id
            )
            .execute(&pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            Ok(Json("Application submitted"))
        }
        Err(e) => {
            if e.to_string().contains("UNIQUE constraint failed") {
                Err(AppError::BadRequest(
                    "You have already applied to this job".into(),
                ))
            } else {
                Err(AppError::Database(e.to_string()))
            }
        }
    }
}
pub async fn approve_application(
    Extension(auth_user): Extension<Arc<AuthUser>>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<ApproveApplicationPayload>,
) -> Result<impl IntoResponse, AppError> {
    // Step 1: Validate client owns the job related to this application
    let record = sqlx::query!(
        r#"
        SELECT ja.user_id as freelancer_id, j.client_id, j.id as job_id, u.wallet_address, p.username
        FROM job_applications ja
        JOIN jobs j ON ja.job_id = j.id
        JOIN users u ON ja.user_id = u.id
        JOIN profiles p ON p.user_id = j.client_id 
        WHERE ja.id = ?
        "#,
        payload.application_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| AppError::NotFound("Application not found".into()))?;

    if record.client_id != auth_user.id {
        return Err(AppError::Unauthorized("User doesn't match.".to_string()));
    }

    // Step 2: Update approval status
    sqlx::query!(
        "UPDATE job_applications SET approved = 1, approved_at =  CURRENT_TIMESTAMP  WHERE id = ?",
        payload.application_id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let msg = format!("{} has approved you application.", record.username);

    // Step 3: Create notification for freelancer
    sqlx::query!(
        "INSERT INTO notifications (user_id, message, read, type, job_id, actor_id) VALUES (?, ?, 0, ?, ?, ?)",
        record.freelancer_id,
        msg,
        "approved",
        record.job_id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({ "message": "Application approved"})))
}

pub async fn create_escrow_notification(
    Extension(auth_user): Extension<Arc<AuthUser>>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateEscrowPayload>,
) -> Result<impl IntoResponse, AppError> {
    // Step 1: Validate client owns the job related to this application
    let record = sqlx::query!(
        r#"
        SELECT ja.user_id as freelancer_id, j.client_id , j.id as job_id, j.title as job_title
        FROM job_applications ja
        JOIN jobs j ON ja.job_id = j.id
        JOIN users u ON ja.user_id = u.id
        WHERE ja.id = ?
        "#,
        payload.application_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| AppError::NotFound("Application not found".into()))?;

    if record.client_id != auth_user.id {
        return Err(AppError::Unauthorized("User doesn't match.".to_string()));
    }

    let msg_client = format!(
        "Escrow {} has be created for job '{}'.",
        payload.escrow_pda, record.job_title
    );

    // Step 3: Create notification for client
    sqlx::query!(
        "INSERT INTO notifications (user_id, message, read, type, job_id, actor_id, escrow_pda) VALUES (?, ?, 0, ?, ?, ?, ?)",
        record.client_id,
        msg_client,
        "escrow",
        record.job_id,
        auth_user.id,
        payload.escrow_pda

    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let msg_freelancer = format!(
        "Escrow {} has be created for the job '{}' you applied.",
        payload.escrow_pda, record.job_title
    );

    sqlx::query!(
        "INSERT INTO notifications (user_id, message, read, type, job_id, actor_id, escrow_pda) VALUES (?, ?, 0, ?, ?, ?, ?)",
        record.freelancer_id,
        msg_freelancer,
        "escrow",
        record.job_id,
        auth_user.id,
        payload.escrow_pda
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({ "message": "Added Notification"})))
}

pub async fn get_job_applicants(
    Path(job_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<ApplicantResponse>>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            ja.id AS application_id,
            ja.user_id AS freelancer_id,
            p.username AS freelancer_username,
            p.skills,
            p.profile_ipfs_hash,
            ja.applied_at,
            ja.approved,
            ja.approved_at,
            ja.freelancer_wallet
        FROM job_applications ja
        JOIN users u ON ja.user_id = u.id
        JOIN profiles p ON p.user_id = u.id
        WHERE ja.job_id = ?
        ORDER BY ja.applied_at DESC
        "#,
        job_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let applicants = rows
        .into_iter()
        .map(|row| ApplicantResponse {
            application_id: row.application_id,
            freelancer_id: row.freelancer_id,
            freelancer_username: row.freelancer_username,
            skills: row.skills,
            profile_ipfs_hash: row.profile_ipfs_hash,
            applied_at: row.applied_at,
            approved: row.approved,
            approved_at: row.approved_at,
            freelancer_wallet: row.freelancer_wallet,
        })
        .collect();

    Ok(Json(applicants))
}
pub async fn get_user_jobs(
    Extension(auth_user): Extension<Arc<AuthUser>>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<MyJobsResponse>>, AppError> {
    let rows = sqlx::query!(
        r#"
SELECT 
    j.id AS job_id,
    j.title,
    j.description,
    j.skills,
    j.budget,
    j.location,
    j.posted_at,
    j.deadline,
    j.client_id,
    ja.applied_at,
    ja.approved,
    CASE WHEN s.job_id IS NOT NULL THEN 1 ELSE 0 END AS is_saved
FROM jobs j
JOIN job_applications ja ON ja.job_id = j.id 
LEFT JOIN saved_jobs s ON s.job_id = j.id AND s.user_id = ja.user_id
WHERE ja.user_id = ?
ORDER BY ja.applied_at DESC;
        "#,
        auth_user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let applicants = rows
        .into_iter()
        .map(|row| MyJobsResponse {
            job_id: row.job_id,
            title: row.title,
            description: row.description,
            skills: row.skills,
            budget: row.budget as u64,
            location: row.location,
            posted_at: row.posted_at,
            deadline: row.deadline,
            client_id: row.client_id,
            applied_at: row.applied_at,
            approved: row.approved,
            is_saved: row.is_saved,
        })
        .collect();

    Ok(Json(applicants))
}

pub async fn get_notifications(
    Extension(auth_user): Extension<Arc<AuthUser>>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            n.id,
            n.message,
            n.read,
            n.created_at,
            n.job_id,
            n.type,
            n.escrow_pda,
            j.title AS job_title,
            pf.username AS username
        FROM notifications n
        LEFT JOIN jobs j ON n.job_id = j.id
        LEFT JOIN profiles pf ON n.actor_id = pf.user_id
        WHERE n.user_id = ?
        ORDER BY n.created_at DESC
        "#,
        auth_user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let notifications: Vec<Notification> = rows
        .into_iter()
        .map(|row| {
            let redirect_url = match row.r#type.as_deref() {
                Some("applied") => format!("/jobs/{}/applicants", row.job_id.unwrap_or_default()),
                Some("approved") => format!("/jobs/{}", row.job_id.unwrap_or_default()),
                Some("escrow") => match &row.escrow_pda {
                    Some(pda) => format!("/escrow/{}", pda),
                    None => "/escrow".to_string(),
                },
                _ => "/".to_string(),
            };

            Notification {
                id: row.id,
                message: row.message,
                read: row.read,
                created_at: row.created_at,
                job_id: row.job_id,
                job_title: row.job_title,
                username: row.username,
                redirect_url: Some(redirect_url),
            }
        })
        .collect();

    Ok(Json(notifications))
}

pub async fn mark_notification_as_read(
    Extension(auth_user): Extension<Arc<AuthUser>>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<MarkReadPayload>,
) -> Result<Json<&'static str>, AppError> {
    let updated = sqlx::query!(
        "UPDATE notifications SET read = 1 WHERE id = ? AND user_id = ?",
        payload.id,
        auth_user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification not found".into()));
    }

    Ok(Json("Notification marked as read"))
}
