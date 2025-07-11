use axum::{Router, routing::{post, get}, middleware};
use crate::handlers::{auth::*};
use sqlx::SqlitePool;
use crate::middleware::auth::{auth_middleware, wallet_verified_only};
use crate::handlers::job::{get_filtered_jobs, get_categories};


pub fn public_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/jobs", get(get_filtered_jobs))
        .route("/jobs/categories", get(get_categories))
        .with_state(pool)
}

pub fn protected_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", post(logout)
            .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware)))
        .route("/profile/basic", 
        get(profile_basic)
                        .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware)))
        .route(
            "/profile/verified",
            get(profile_verified)
                .route_layer(middleware::from_fn_with_state(pool.clone(), wallet_verified_only))
                .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware))
        )
        .route("/wallet/request-nonce", post(request_nonce))
        .route("/wallet/verify", post(verify))
        .with_state(pool)
}