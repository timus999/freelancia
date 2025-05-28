use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use freelancia_backend::{
    handlers::auth::{profile_basic, profile_verified},
    middleware::auth::{auth_middleware, wallet_verified_only},
    models::auth::*,
    utils::generate_jwt,
};
use sqlx::{Pool, Sqlite};
use tower::util::ServiceExt;

async fn setup_db() -> Pool<Sqlite> {
    let pool = sqlx::sqlite::SqlitePool::connect(":memory:").await.unwrap();
    sqlx::query!(
        r#"
    CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    password TEXT, -- Hashed, nullable for Web3 users
    wallet_address TEXT UNIQUE,
    role TEXT NOT NULL,
    verified_wallet BOOLEAN NOT NULL DEFAULT FALSE
    );
      CREATE TABLE IF NOT EXISTS nonces (
    wallet_address TEXT NOT NULL,
    nonce TEXT NOT NULL,
    created_at TEXT NOT NULL, -- ISO 8601
    expires_at TEXT NOT NULL, -- ISO 8601
    PRIMARY KEY (wallet_address, nonce)
    );
        "#
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn test_profile_basic_email_user() {
    let pool = setup_db().await;

    sqlx::query!(
        "INSERT INTO users (email, password, role, verified_wallet) VALUES (?, ?, ?, ?)",
        "email@example.com",
        "hashed_password",
        "email_user",
        false
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = Router::new()
        .route("/auth/profile/basic", get(profile_basic))
        .route_layer(axum::middleware::from_fn_with_state(pool.clone(), auth_middleware))
        .with_state(pool);

    let token = generate_jwt(1, "email_user".to_string()).unwrap();
    let request = Request::builder()
        .method("GET")
        .uri("/auth/profile/basic")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let profile: ProfileResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(profile.email, "email@example.com");
    assert_eq!(profile.role, "email_user");
    assert_eq!(profile.wallet_address, None);
    assert_eq!(profile.verified_wallet, false);
}

#[tokio::test]
async fn test_profile_verified_unverified_wallet_user() {
    let pool = setup_db().await;

    sqlx::query!(
        "INSERT INTO users (email, password, wallet_address, role, verified_wallet) VALUES (?, ?, ?, ?, ?)",
        "wallet@example.com",
        "hashed_password",
        "0x1234567890abcdef1234567890abcdef12345678",
        "wallet_user",
        false
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = Router::new()
        .route("/profile/verified", get(profile_verified))
        .route_layer(axum::middleware::from_fn_with_state(pool.clone(), wallet_verified_only))
        .route_layer(axum::middleware::from_fn_with_state(pool.clone(), auth_middleware))
        .with_state(pool);

    let token = generate_jwt(1, "wallet_user".to_string()).unwrap();
    let request = Request::builder()
        .method("GET")
        .uri("/profile/verified")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"{\"error\":\"Wallet verification required\"}");
}

#[tokio::test]
async fn test_profile_verified_verified_client() {
    let pool = setup_db().await;

    sqlx::query!(
        "INSERT INTO users (email, password, wallet_address, role, verified_wallet) VALUES (?, ?, ?, ?, ?)",
        "client@example.com",
        "hashed_password",
        "0x1234567890abcdef1234567890abcdef12345678",
        "client",
        true
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = Router::new()
        .route("/profile/verified", get(profile_verified))
        .route_layer(axum::middleware::from_fn_with_state(pool.clone(), wallet_verified_only))
        .route_layer(axum::middleware::from_fn_with_state(pool.clone(), auth_middleware))
        .with_state(pool);

    let token = generate_jwt(1, "client".to_string()).unwrap();
    let request = Request::builder()
        .method("GET")
        .uri("/profile/verified")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let profile: ProfileResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(profile.email, "client@example.com");
    assert_eq!(profile.role, "client");
    assert_eq!(profile.wallet_address, Some("0x1234567890abcdef1234567890abcdef12345678".to_string()));
    assert_eq!(profile.verified_wallet, true);
}