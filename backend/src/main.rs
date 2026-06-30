use axum::extract::Query;
use axum::{Router, http::StatusCode, routing::get};
use dotenvy::dotenv;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, error, info};

use tracing_subscriber::{EnvFilter, prelude::*};

use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;

use std::env;

use serde::Deserialize;

#[tokio::main]
async fn main() {
    let _env = dotenv().unwrap();

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("apu_backend=info,tower_http=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let middleware_logging = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::DEBUG));

    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/send_mail", get(send_mail))
        .layer(middleware_logging);

    let addr = "127.0.0.1:8080";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Apu server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn handle_health() -> StatusCode {
    StatusCode::OK
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
