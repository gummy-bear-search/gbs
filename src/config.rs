use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Elasticsearch compatibility version (default: "6.4.0")
    #[serde(default = "default_es_version")]
    pub es_version: String,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ServerConfig {
    /// Server host (default: "0.0.0.0")
    #[serde(default = "default_host")]
    pub host: String,
    /// Server port (default: 9200)
    #[serde(default = "default_port")]
    pub port: u16,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StorageConfig {
    /// Data directory path (default: "./data")
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LoggingConfig {
    /// Log level (default: "info")
    /// Valid values: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    9200
}

fn default_data_dir() -> String {
    "./data".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_es_version() -> String {
    "6.4.0".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
            },
            storage: StorageConfig {
                data_dir: default_data_dir(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
            },
            es_version: default_es_version(),
        }
    }
}

impl Config {
    /// Load configuration from file or environment variables
    ///
    /// Priority order:
    /// 1. Environment variables (highest priority)
    /// 2. Config file (if exists)
    /// 3. Default values (lowest priority)
    pub fn load() -> anyhow::Result<Self> {
        // Try to load from config file
        let config = Self::load_from_file().unwrap_or_else(|e| {
            warn!("Failed to load config file: {}. Using defaults.", e);
            Config::default()
        });

        // Override with environment variables
        let config = config.with_env_overrides();

        info!("Loaded configuration: server={}:{}, data_dir={}, log_level={}, es_version={}",
              config.server.host,
              config.server.port,
              config.storage.data_dir,
              config.logging.level,
              config.es_version);

        Ok(config)
    }

    /// Load configuration from YAML file
    ///
    /// Tries to load from:
    /// 1. GUMMY_CONFIG environment variable (if set)
    /// 2. ./gummy-search.yaml
    /// 3. ./config/gummy-search.yaml
    /// 4. ~/.config/gummy-search/gummy-search.yaml
    fn load_from_file() -> anyhow::Result<Self> {
        let config_paths = vec![
            std::env::var("GUMMY_CONFIG")
                .ok()
                .map(PathBuf::from),
            Some(PathBuf::from("./gummy-search.yaml")),
            Some(PathBuf::from("./config/gummy-search.yaml")),
            dirs::home_dir()
                .map(|mut p| {
                    p.push(".config");
                    p.push("gummy-search");
                    p.push("gummy-search.yaml");
                    p
                }),
        ];

        for path in config_paths.into_iter().flatten() {
            if path.exists() {
                info!("Loading config from: {}", path.display());
                let content = std::fs::read_to_string(&path)?;
                let config: Config = serde_yaml::from_str(&content)?;
                return Ok(config);
            }
        }

        anyhow::bail!("No config file found")
    }

    /// Apply environment variable overrides
    pub fn with_env_overrides(mut self) -> Self {
        // Server host
        if let Ok(host) = std::env::var("GUMMY_HOST") {
            self.server.host = host;
        }

        // Server port
        if let Ok(port_str) = std::env::var("GUMMY_PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                self.server.port = port;
            } else {
                warn!("Invalid GUMMY_PORT value: {}. Using default.", port_str);
            }
        }

        // Data directory
        if let Ok(data_dir) = std::env::var("GUMMY_DATA_DIR") {
            self.storage.data_dir = data_dir;
        }

        // Log level (RUST_LOG takes precedence if set)
        if std::env::var("RUST_LOG").is_ok() {
            // RUST_LOG is handled by tracing_subscriber, so we don't override here
            // But we can still set it in the config for reference
        } else if let Ok(level) = std::env::var("GUMMY_LOG_LEVEL") {
            self.logging.level = level;
        }

        // Elasticsearch version
        if let Ok(es_version) = std::env::var("GUMMY_ES_VERSION") {
            self.es_version = es_version;
        }

        self
    }

    /// Get server address as SocketAddr
    pub fn server_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from((
            self.server.host.parse::<std::net::IpAddr>()
                .unwrap_or_else(|_| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))),
            self.server.port,
        ))
    }
}
