use axum::Json;
use axum::extract::{Query, State};
use axum::routing::post;
use axum::{Router, http::StatusCode, routing::get};
use dotenvy::dotenv;
use sqlx::PgPool;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, error, info, warn};

use tracing_subscriber::{EnvFilter, prelude::*};

use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;

use sqlx::postgres::PgPoolOptions;

use std::env;
use std::sync::Arc;

use serde::Deserialize;

use crate::config::AppConfig;
use crate::domains::auth::middleware::CurrentUser;

use axum::extract::FromRef;

use crate::shared::email::{EmailService, ResendEmailService};

mod config;
pub mod domains;
mod shared;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub email_service: Arc<dyn EmailService + Send + Sync>,
}

impl axum::extract::FromRef<AppState> for sqlx::PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load().map_err(|e| {
        error!("Configuration error: {}", e);
        e
    })?;

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url)
        .await?;

    info!(
        "Succesfully connected to database '{}'",
        config.database.db_name
    );

    sqlx::migrate!("./migrations").run(&db_pool).await?;

    info!("Succesfully ran migrations");

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("apu_backend=info,tower_http=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let middleware_logging = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::DEBUG));

    let email_service = Arc::new(ResendEmailService::new(&config.resend_api_key));

    let state = AppState {
        db_pool,
        email_service,
    };

    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/send_mail", get(send_mail))
        .route("/inbound", post(handle_inbound_webhook))
        .route("/test_auth", get(test_auth))
        .nest("/auth", domains::auth::routes::router())
        .with_state(state)
        .layer(middleware_logging);

    let addr = "127.0.0.1:8080";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Apu server running on http://{}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}

async fn handle_health() -> StatusCode {
    StatusCode::OK
}

async fn test_auth(user: CurrentUser) -> Result<StatusCode, StatusCode> {
    info!("User '{}' authenticated", user.username);

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct MailQueryParams {
    to: String,
}

async fn send_mail(
    Query(params): Query<MailQueryParams>,
) -> Result<StatusCode, (StatusCode, String)> {
    let resend = Resend::default();

    let from = env::var("SENDER_EMAIL").unwrap();
    let to = [params.to.as_str()];

    let subject = "Apu Backend Verification Link";
    let html_body = format!(
        "<h2>Greetings!</h2><p>This test email was triggered dynamically for <strong>{}</strong>.</p>",
        params.to
    );

    info!("Dispatched dynamic URL test mail to: {:?}", to);

    resend
        .emails
        .send(CreateEmailBaseOptions::new(&from, to, subject).with_html(&html_body))
        .await
        .map_err(|e| {
            error!("Failed to send email via Resend: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Email error: {:?}", e),
            )
        })?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Debug)]
struct ResendWebhookEvent {
    r#type: String,
    created_at: String,
    data: InboundEmailMetadata,
}

#[derive(Deserialize, Debug)]
struct InboundEmailMetadata {
    email_id: String,
    from: String,
    to: Vec<String>,
    subject: Option<String>,
}

async fn handle_inbound_webhook(Json(payload): Json<ResendWebhookEvent>) -> StatusCode {
    info!("Webhook endpoint executed!");

    if payload.r#type != "email.received" {
        warn!("Ignored unsupported webhook event type: {}", payload.r#type);
        return StatusCode::OK;
    }

    let email = payload.data;
    info!("----------------------------------------");
    info!("New Mail Received via Resend!");
    info!("From: {}", email.from);
    info!("To Array: {:?}", email.to);
    info!(
        "Subject: {}",
        email.subject.unwrap_or_else(|| "No Subject".to_string())
    );
    info!("Resend Inbound Email ID Reference: {}", email.email_id);
    info!("----------------------------------------");

    StatusCode::OK
}
