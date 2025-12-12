use gummy_search::server::{create_app, AppState};
use gummy_search::storage::Storage;
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create storage with Sled persistence
    // Use data directory from environment or default to ./data
    let data_dir = std::env::var("GUMMY_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    tracing::info!("Using data directory: {}", data_dir);

    let storage = Storage::with_sled(&data_dir)?;
    storage.load_from_backend().await?;

    let state = AppState {
        storage: std::sync::Arc::new(storage),
    };

    // Create app
    let app = create_app(state).await;

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 9200));
    tracing::info!("Gummy Search server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
