//! Unit tests for Config module

use gummy_search::config::Config;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9200);
    assert_eq!(config.storage.data_dir, "./data");
    assert_eq!(config.logging.level, "info");
    assert_eq!(config.es_version, "6.8.23");
}

#[test]
fn test_env_overrides() {
    std::env::set_var("GUMMY_PORT", "9300");
    std::env::set_var("GUMMY_DATA_DIR", "/tmp/test");
    std::env::set_var("GUMMY_ES_VERSION", "7.0.0");

    let config = Config::default().with_env_overrides();

    assert_eq!(config.server.port, 9300);
    assert_eq!(config.storage.data_dir, "/tmp/test");
    assert_eq!(config.es_version, "7.0.0");

    // Cleanup
    std::env::remove_var("GUMMY_PORT");
    std::env::remove_var("GUMMY_DATA_DIR");
    std::env::remove_var("GUMMY_ES_VERSION");
}
