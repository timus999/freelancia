use axum::{Json, response::IntoResponse};
use serde_json::json;
use crate::models::auth::AuthUser;

pub async fn profile_handler(user: AuthUser) -> impl IntoResponse{
    Json(json!({
        "message" : "Welcome to your profile",
        "user_id": user.user_id
    }))
}