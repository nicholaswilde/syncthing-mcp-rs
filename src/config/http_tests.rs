#[cfg(test)]
mod tests {
    use crate::config::{AppConfig, ConfigResult};
    use crate::test_utils::ENV_LOCK;
    use std::io::Write;

    #[tokio::test]
    async fn test_http_server_config_loading() {
        let _guard = ENV_LOCK.lock().await;
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"
host = "localhost"
port = 8384
api_key = "test"

[http_server]
enabled = true
host = "127.0.0.1"
port = 3001
api_key = "http-secret"
"#
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = match AppConfig::load(Some(path), vec![]).await.unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert!(config.http_server.enabled);
        assert_eq!(config.http_server.host, "127.0.0.1");
        assert_eq!(config.http_server.port, 3001);
        assert_eq!(config.http_server.api_key, Some("http-secret".to_string()));
    }

    #[tokio::test]
    async fn test_http_server_config_defaults() {
        let _guard = ENV_LOCK.lock().await;
        let config = match AppConfig::load(None, vec![]).await.unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert!(!config.http_server.enabled);
        assert_eq!(config.http_server.host, "0.0.0.0");
        assert_eq!(config.http_server.port, 3000);
        assert!(config.http_server.api_key.is_none());
    }

    #[tokio::test]
    async fn test_http_server_cli_override() {
        let _guard = ENV_LOCK.lock().await;
        let args = vec![
            "app".to_string(),
            "--http-enabled".to_string(),
            "--http-host".to_string(),
            "1.1.1.1".to_string(),
            "--http-port".to_string(),
            "9999".to_string(),
            "--http-api-key".to_string(),
            "cli-secret".to_string(),
        ];
        let config = match AppConfig::load(None, args).await.unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert!(config.http_server.enabled);
        assert_eq!(config.http_server.host, "1.1.1.1");
        assert_eq!(config.http_server.port, 9999);
        assert_eq!(config.http_server.api_key, Some("cli-secret".to_string()));
    }

    #[tokio::test]
    async fn test_mcp_events_config_loading() {
        let _guard = ENV_LOCK.lock().await;
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"
host = "localhost"
port = 8384
api_key = "test"
mcp_events = ["FolderSummary", "StateChanged"]
"#
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = match AppConfig::load(Some(path), vec![]).await.unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert_eq!(config.mcp_events.len(), 2);
        assert!(config.mcp_events.contains(&"FolderSummary".to_string()));
        assert!(config.mcp_events.contains(&"StateChanged".to_string()));
    }

    #[tokio::test]
    async fn test_mcp_events_defaults() {
        let _guard = ENV_LOCK.lock().await;
        let config = match AppConfig::load(None, vec![]).await.unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert!(
            config
                .mcp_events
                .contains(&"FolderStateChanged".to_string())
        );
        assert!(config.mcp_events.contains(&"DeviceConnected".to_string()));
    }
}
