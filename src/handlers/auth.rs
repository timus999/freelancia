use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;
use uuid::Uuid;
use sqlx::SqlitePool;
use crate::models::auth::LoginInput;
use crate::models::user::{User, SignupInput};
use crate::utils::hash_password;
use crate::utils::{verify_password, generate_jwt};



pub async fn signup(
    State(pool): State<SqlitePool>,
    Json(payload): Json<SignupInput>,
) -> Result<Json<User>, StatusCode>{
    let hashed = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    };

    let user = User {
        id: Uuid::new_v4().to_string(),
        email: payload.email.clone(),
        password_hash: hashed,
    };

    // Insert user into DB
    let result = sqlx::query("INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)")
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .execute(&pool)
        .await;
    if let Err(e) = result {
        eprint!("Failed to insert user: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::Json(user))
}


//login logic
pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginInput>,
) -> Result<Json<serde_json::Value>, StatusCode> {

    //fetch user by email
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash FROM users WHERE email = ?
        "#,
        payload.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    //check if user exist
    let user = match user {
        Some(u) => u,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    //verify password
    if verify_password(&payload.password, &user.password_hash).is_err(){
        return Err(StatusCode::UNAUTHORIZED);
    }

    //generate JWT token with string ID
    let token = generate_jwt(&user.id.to_string()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // return token as JSON
    Ok(Json(json!({ "token" : token })))

}