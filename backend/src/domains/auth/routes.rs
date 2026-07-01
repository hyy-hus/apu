use axum::routing::post;
use axum::{Json, Router, extract::State};
use reqwest::StatusCode;
use sqlx::PgPool;

use crate::AppState;

use crate::domains::auth::{hash_password, schema::RegisterUser};

async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterUser>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let hashed_password = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
    })?;

    let result = sqlx::query!(
        r#"
        INSERT INTO users (username, email, name, password_hash)
        VALUES ($1, $2, $3, $4)
        "#,
        payload.username,
        payload.email,
        payload.name,
        hashed_password
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Username or email already exists" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Database error: {}", e) })),
        )),
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/register", post(register))
}
