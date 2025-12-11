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

    // Create storage
    let storage = Storage::new();
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
