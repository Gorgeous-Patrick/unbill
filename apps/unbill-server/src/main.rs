use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::info;

use unbill_core::net::UnbillEndpoint;
use unbill_core::storage::LedgerStore;
use unbill_server::{AppState, build_router};
use unbill_store_fs::FsStore;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("API_KEY").context("API_KEY must be set")?;
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_owned())
        .parse()
        .context("PORT must be a valid port number")?;

    let data_dir = unbill_store_fs::UNBILL_PATH
        .ensure_data_dir()
        .context("failed to resolve data directory")?;
    let store = Arc::new(FsStore::new(data_dir));

    let endpoint = if store.is_device_initialized().await? {
        let key = store.get_secret_key().await?;
        Some(Arc::new(UnbillEndpoint::bind(&key).await?))
    } else {
        info!("no device key found; peer sync disabled until POST /device/key is called");
        None
    };

    let (events, _) = broadcast::channel(16);
    let store: Arc<dyn LedgerStore> = store;
    let state = Arc::new(AppState {
        store,
        api_key,
        endpoint,
        events,
    });
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
