use crate::handlers::{auth::*, escrow::*, job::*, profile::*};

use crate::middleware::auth::{auth_middleware, wallet_verified_only};
use axum::{
    middleware,
    routing::{get, post},
    Extension, Router,
};
use sqlx::SqlitePool;

pub fn public_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/jobs",
            get(get_filtered_jobs).route_layer(middleware::from_fn_with_state(
                pool.clone(),
                auth_middleware,
            )),
        )
        .route(
            "/jobs/categories",
            get(get_categories).route_layer(middleware::from_fn_with_state(
                pool.clone(),
                auth_middleware,
            )),
        )
        .route(
            "/notifications/mark-read",
            post(mark_notification_as_read).route_layer(middleware::from_fn_with_state(
                pool.clone(),
                auth_middleware,
            )),
        )
        .with_state(pool)
}

pub fn auth_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/signup", post(signup))
        .route("/wallet/signup", post(wallet_signup))
        .route("/login", post(login))
        .route("/wallet/login", post(wallet_login))
        .route("/wallet/request-nonce", post(request_nonce))
        .route("/wallet/verify", post(verify))
        .with_state(pool)
}

pub fn protected_routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/wallet/connect", post(wallet_connect))
        .route("/logout", post(logout))
        .route("/notifications", get(get_notifications))
        .route("/profile", post(create_or_update_profile))
        .route("/get-profile-userId/:user_id", get(get_profile_by_user_id))
        .route(
            "/get-profile-username/:username",
            get(get_profile_by_username),
        )
        .route("/username-availability", get(check_username_availability))
        .route("/profile/basic", get(check_username_availability))
        .route("/escrow/:escrow_pda", get(get_escrow))
        .route("/my-jobs", get(get_user_jobs))
        .route("/raise-dispute", post(raise_dispute))
        .route("/get-disputed-jobs", get(get_disputed_jobs_for_arbiter))
        .route("/handle-resolve", post(arbiter_resolve))
        .route(
            "/profile/verified",
            get(profile_verified).route_layer(middleware::from_fn_with_state(
                pool.clone(),
                wallet_verified_only,
            )),
        )
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ))
        .layer(Extension(pool.clone()))
        .with_state(pool)
}
