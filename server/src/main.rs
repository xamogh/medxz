#![forbid(unsafe_code)]

use std::net::SocketAddr;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .with_target(false)
        .init();

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:1426".into())
        .parse()
        .expect("invalid BIND_ADDR");

    let app = medxz_server::app::router();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind BIND_ADDR");
    tracing::info!(%addr, "server listening");

    axum::serve(listener, app).await.expect("server error");
}
