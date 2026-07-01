use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterUser {
    pub username: String,
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: UserClaims,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub id: Uuid,
    pub username: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: Uuid,
    pub exp: i64,
    pub token_version: i32,
    pub user: UserClaims,
}
