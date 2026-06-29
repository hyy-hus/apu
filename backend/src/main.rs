use axum::{Router, http::StatusCode, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health", get(handle_health));

    let addr = "127.0.0.1:8080";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn handle_health() -> StatusCode {
    StatusCode::OK
}
