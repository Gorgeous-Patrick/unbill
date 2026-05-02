use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tracing::info;

use unbill_server::{AppState, build_router};
use unbill_store_fs::FsStore;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("API_KEY").context("API_KEY must be set")?;
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_owned());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_owned())
        .parse()
        .context("PORT must be a valid port number")?;

    let store = FsStore::new(data_dir.into());
    let state = Arc::new(AppState { store, api_key });
    let router = build_router(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;

    info!("unbill-server listening on {addr}");
    axum::serve(listener, router)
        .await
        .context("server error")?;

    Ok(())
}
