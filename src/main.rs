use gummy_search::config::Config;
use gummy_search::server::{create_app, AppState};
use gummy_search::storage::Storage;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Initialize tracing
    // RUST_LOG environment variable takes precedence over config
    let log_filter = if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::EnvFilter::from_default_env()
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.logging.level))
    };

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .init();

    tracing::info!("Starting Gummy Search server");
    tracing::info!("Configuration: server={}:{}, data_dir={}, log_level={}",
                   config.server.host,
                   config.server.port,
                   config.storage.data_dir,
                   config.logging.level);

    // Create storage with Sled persistence
    let storage = Storage::with_sled(&config.storage.data_dir)?;
    storage.load_from_backend().await?;

    let state = AppState {
        storage: std::sync::Arc::new(storage),
    };

    // Create app
    let app = create_app(state).await;

    // Start server
    let addr = config.server_addr();
    tracing::info!("Gummy Search server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
