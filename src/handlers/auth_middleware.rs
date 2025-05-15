use axum::{
    async_trait,
    extract::{FromRequestParts},
    http::{request::Parts, StatusCode}
};

use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;
use crate::models::jwt::Claims;
use crate::models::auth::AuthUser;




#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where 
S: Send + Sync,
{

    type Rejection = StatusCode; // this is required for axum 0.7+
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, StatusCode>{
        let auth_header = parts.headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
        let token = auth_header.strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

        let secret = env::var("JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let token_data = decode::<Claims>(
            token, 
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        ).map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
        })
        }
    }

