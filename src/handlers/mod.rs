
use axum::{Json};
use axum::response::IntoResponse;
use serde_json::json;


pub mod auth;
pub mod protected;
pub mod freelancer;
pub mod job;
pub mod proposal;

pub async fn check_health() -> impl IntoResponse {
    Json(json!({"status" : "ok"}))
}

pub async fn print_msg() -> impl IntoResponse {
    "This is freelancia backend"
}

pub async fn hello() -> impl IntoResponse{
    "hello from freelancia api"

}


