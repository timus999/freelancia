use axum::{Router, routing::{post, get}, middleware};
use crate::handlers::{auth::*};
use sqlx::SqlitePool;
use crate::middleware::auth::{auth_middleware, wallet_verified_only};

pub fn protected_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
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