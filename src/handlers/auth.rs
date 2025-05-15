use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;
use sqlx::SqlitePool;

use crate::models::user::{User, SignupInput};
use crate::utils::hash_password;



pub async fn signup(
    State(pool): State<SqlitePool>,
    Json(payload): Json<SignupInput>,
) -> Result<Json<User>, StatusCode>{
    let hashed = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    };

    let user = User {
        id: Uuid::new_v4(),
        email: payload.email.clone(),
        password_hash: hashed,
    };

    // Insert user into DB
    let result = sqlx::query("INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)")
        .bind(user.id)
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
