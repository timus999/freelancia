use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

pub mod auth;
pub mod client;
pub mod escrow;
pub mod freelancer;
pub mod job;
pub mod profile;
pub mod proposal;
pub mod protected;
pub async fn check_health() -> impl IntoResponse {
    Json(json!({"status" : "ok"}))
}

pub async fn print_msg() -> impl IntoResponse {
    "This is freelancia backend"
}

pub async fn hello() -> impl IntoResponse {
    "hello from freelancia api"
}
