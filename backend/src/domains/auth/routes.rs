use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode, header};
use axum::routing::post;
use axum::{Json, Router, extract::State};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use sqlx::types::ipnetwork::IpNetwork;
use uuid::Uuid;

use crate::AppState;
use crate::domains::auth::hash_password;
use crate::domains::auth::schema::{AuthResponse, Login, RegisterUser, TokenClaims, UserClaims};
use crate::domains::auth::verify_password;

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

async fn login(
    State(pool): State<PgPool>,
    jar: CookieJar,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<Login>,
) -> Result<(CookieJar, Json<AuthResponse>), StatusCode> {
    let user = sqlx::query!(
        r#"
        SELECT id, username, password_hash, role::text as "role!", token_version 
        FROM users 
        WHERE username = $1 AND deleted_at IS NULL
        "#,
        payload.username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let is_valid = verify_password(&payload.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !is_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let ip_network = IpNetwork::from(addr.ip());
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown Device")
        .to_string();

    let now = Utc::now();
    let access_expiry = now + TimeDelta::minutes(15);
    let refresh_expiry = now + TimeDelta::days(7);

    let raw_refresh_token = Uuid::new_v4().to_string();
    let mut hasher = Sha256::new();
    hasher.update(raw_refresh_token.as_bytes());

    let token_hash = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, ip_address, user_agent, token_hash, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        user.id,
        ip_network,
        user_agent,
        token_hash,
        refresh_expiry
    )
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let claims = TokenClaims {
        sub: user.id,
        exp: access_expiry.timestamp(),
        token_version: user.token_version,
        user: UserClaims {
            id: user.id,
            username: user.username,
            role: user.role,
        },
    };

    let jwt_secret = b"your-temporary-super-secret-development-key-12345";

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie_expires = time::OffsetDateTime::from_unix_timestamp(refresh_expiry.timestamp())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refresh_cookie = Cookie::build(("refresh_token", raw_refresh_token))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .expires(cookie_expires)
        .build();

    let updated_jar = jar.add(refresh_cookie);

    Ok((
        updated_jar,
        Json(AuthResponse {
            access_token,
            user: claims.user,
        }),
    ))
}

async fn refresh(
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<AuthResponse>), StatusCode> {
    let raw_refresh_token = jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let mut hasher = Sha256::new();
    hasher.update(raw_refresh_token.as_bytes());
    let incoming_token_hash = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let session_context = sqlx::query!(
        r#"
        SELECT 
            s.id as session_id,
            s.user_id,
            u.username,
            u.role::text as "role!",
            u.token_version
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.token_hash = $1 
          AND s.is_revoked = FALSE 
          AND s.expires_at > now()
          AND u.deleted_at IS NULL
        "#,
        incoming_token_hash
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let now = Utc::now();
    let access_expiry = now + TimeDelta::minutes(15);
    let refresh_expiry = now + TimeDelta::days(7);

    let new_raw_refresh_token = Uuid::new_v4().to_string();
    let mut new_hasher = Sha256::new();
    new_hasher.update(new_raw_refresh_token.as_bytes());
    let new_token_hash = new_hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    sqlx::query!(
        r#"
        UPDATE sessions
        SET token_hash = $1, last_active_at = now(), expires_at = $2
        WHERE id = $3
        "#,
        new_token_hash,
        refresh_expiry,
        session_context.session_id
    )
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let claims = TokenClaims {
        sub: session_context.user_id,
        exp: access_expiry.timestamp(),
        token_version: session_context.token_version,
        user: UserClaims {
            id: session_context.user_id,
            username: session_context.username,
            role: session_context.role,
        },
    };

    let jwt_secret = b"your-temporary-super-secret-development-key-12345";
    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie_expires = time::OffsetDateTime::from_unix_timestamp(refresh_expiry.timestamp())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refresh_cookie = Cookie::build(("refresh_token", new_raw_refresh_token))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .expires(cookie_expires)
        .build();

    let updated_jar = jar.add(refresh_cookie);

    Ok((
        updated_jar,
        Json(AuthResponse {
            access_token,
            user: claims.user,
        }),
    ))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
}
