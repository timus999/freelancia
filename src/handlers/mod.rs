use std::vec;

use axum::{http::StatusCode, Json};
use axum::response::IntoResponse;
use serde_json::json;


use crate::models::{Job, Freelancer};

pub mod auth;
pub mod profile;
pub mod auth_middleware;

pub async fn check_health() -> impl IntoResponse {
    Json(json!({"status" : "ok"}))
}

pub async fn print_msg() -> impl IntoResponse {
    "This is freelancia backend"
}

pub async fn hello() -> impl IntoResponse{
    "hello from freelancia api"

}

pub async fn get_jobs() -> Json<Vec<Job>> {
    let jobs = vec![
        Job{
            id:1,
            title:"Build a Rust web app".to_string(),
            budget: 500,
            description: "Need a Rust developer for a small backend project.".to_string(),
        },
        Job{
            id: 2,
            title: "Write blockchain smart contract".to_string(),
            budget: 800,
            description: "Solidity-based smart contract for escrow system.".to_string(),
        },
    ];
    Json(jobs)
}

pub async fn get_freelancers() -> Json<Vec<Freelancer>> {
    let freelancer = vec![
        Freelancer{
            id: 1,
            name: "Alice Dev".to_string(),
            skills: vec!["Rust".to_string(), "blockchain".to_string()],
            rating: 4.8,
        },
        Freelancer{
            id: 2,
            name: "Bob Coder".to_string(),
            skills: vec!["Solidity".to_string(), "Ethereum".to_string()],
            rating: 4.5,
        },
    ];
    Json(freelancer)
}

pub async fn create_job(Json(payload): Json<Job>) -> (StatusCode, Json<Job>){
    // late save to db
    (StatusCode::CREATED, Json(payload))
}

pub async fn create_freelancer(Json(payload): Json<Freelancer>) -> (StatusCode, Json<Freelancer>){
    (StatusCode::CREATED, Json(payload))
}

