use axum::extract::FromRequestParts;
use jsonwebtoken::{DecodingKey, Validation, decode};
use reqwest::{StatusCode, header};
use uuid::Uuid;

use crate::domains::auth::schema::TokenClaims;

pub struct CurrentUser {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    pub token_version: i32,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];

        let jwt_secret = b"your-temporary-super-secret-development-key-12345";

        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(jwt_secret),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(CurrentUser {
            id: token_data.claims.sub,
            username: token_data.claims.user.username,
            role: token_data.claims.user.role,
            token_version: token_data.claims.token_version,
        })
    }
}
