#![forbid(unsafe_code)]

use std::net::SocketAddr;

use thiserror::Error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    if let Err(err) = run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[derive(Debug, Error)]
enum ServerError {
    #[error("invalid BIND_ADDR {value}: {source}")]
    InvalidBindAddr {
        value: String,
        source: std::net::AddrParseError,
    },
    #[error(transparent)]
    Db(#[from] medxz_server::db::DbError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

async fn run() -> Result<(), ServerError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();

    let addr_raw = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:1426".into());
    let addr: SocketAddr = addr_raw
        .parse()
        .map_err(|source| ServerError::InvalidBindAddr {
            value: addr_raw,
            source,
        })?;

    let pool = medxz_server::db::connect_from_env_and_migrate().await?;
    let app_state = medxz_server::state::AppState { pool };
    let app = medxz_server::app::router(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "server listening");

    axum::serve(listener, app).await?;
    Ok(())
}
