

// use axum::{
//     extract::{Path, State},
//     Json,
//     http::StatusCode,
//     response::IntoResponse,
// };
// use serde_json::json;
// use sqlx::SqlitePool;
// use validator::{Validate};
// use crate::models::auth::AuthUser;

// use crate::models::bid::BidRequest;

// pub async fn submit_bid(
//     AuthUser { user_id }: AuthUser,
//     State(pool): State<SqlitePool>,
//     Path(job_id): Path<u64>,
//     Json(payload): Json<BidRequest>,
// ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
//     if let Err(e) = payload.validate() {
//         return Err((
//             StatusCode::BAD_REQUEST,
//             Json(json!({ "error": "Validation failed", "details": e })),
//         ));
//     }

//     if payload.job_id != job_id {
//         return Err((
//             StatusCode::BAD_REQUEST,
//             Json(json!({ "error": "Job ID in path and body must match" })),
//         ));
//     }

//     let job_id = payload.job_id as i64;
//     let budget = payload.budget as i64;
//     let timeline = payload.timeline.clone();
//     let message = payload.message.clone();

//     let res = sqlx::query!(
//         r#"
//         INSERT INTO bids (job_id, freelancer_id, timeline, budget, message)
//         VALUES (?, ?, ?, ?, ?)
//         "#,
//         job_id,
//         user_id,
//         budget,
//         timeline,
//         message
//     )
//     .execute(&pool)
//     .await;

//     match res {
//         Ok(result) => {
//             let bid_id = result.last_insert_rowid();
//             Ok((
//                 StatusCode::OK,
//                 Json(json!({ "message": "Bid submitted", "bid_id": bid_id })),
//             ))
//         }
//         Err(e) => Err((
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({ "error": "Failed to insert bid", "details": e.to_string() })),
//         )),
//     }
// }