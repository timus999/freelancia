use axum::{Router, routing::{post, get}, middleware};
use crate::handlers::{escrow::get_idl};
use sqlx::SqlitePool;
use crate::middleware::auth::{auth_middleware, wallet_verified_only};

pub fn on_chain_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/create-escrow", get(get_idl))
        .with_state(pool)

}