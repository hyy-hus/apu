use axum::{Router, http::StatusCode, routing::get};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, info};

use tracing_subscriber::{EnvFilter, prelude::*};

#[tokio::main]
async fn main() {
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
        .layer(middleware_logging);

    let addr = "127.0.0.1:8080";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Apu server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn handle_health() -> StatusCode {
    StatusCode::OK
}
