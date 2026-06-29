use axum::{Router, http::StatusCode, routing::get};
use tracing::info;

use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/health", get(handle_health));

    let addr = "127.0.0.1:8080";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Apu server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn handle_health() -> StatusCode {
    StatusCode::OK
}
