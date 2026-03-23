#[cfg(test)]
mod tests {
    use crate::config::{AppConfig, ConfigResult};
    use crate::test_utils::ENV_LOCK;
    use std::io::Write;

    #[test]
    fn test_http_server_config_loading() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"
[http_server]
enabled = true
host = "127.0.0.1"
port = 8080
"#
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = match AppConfig::load(Some(path), vec![]).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert!(config.http_server.enabled);
        assert_eq!(config.http_server.host, "127.0.0.1");
        assert_eq!(config.http_server.port, 8080);
    }

    #[test]
    fn test_http_server_config_defaults() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let config = match AppConfig::load(None, vec![]).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert!(!config.http_server.enabled);
        assert_eq!(config.http_server.host, "0.0.0.0");
        assert_eq!(config.http_server.port, 3000);
    }

    #[test]
    fn test_http_server_cli_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let args = vec![
            "app".to_string(),
            "--http-enabled".to_string(),
            "--http-host".to_string(),
            "1.2.3.4".to_string(),
            "--http-port".to_string(),
            "9090".to_string(),
        ];
        let config = match AppConfig::load(None, args).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert!(config.http_server.enabled);
        assert_eq!(config.http_server.host, "1.2.3.4");
        assert_eq!(config.http_server.port, 9090);
    }

    #[test]
    fn test_mcp_events_config_loading() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"
mcp_events = ["FolderStateChanged", "DeviceConnected"]
"#
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = match AppConfig::load(Some(path), vec![]).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert_eq!(config.mcp_events.len(), 2);
        assert!(config.mcp_events.contains(&"FolderStateChanged".to_string()));
        assert!(config.mcp_events.contains(&"DeviceConnected".to_string()));
    }

    #[test]
    fn test_mcp_events_defaults() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let config = match AppConfig::load(None, vec![]).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        assert!(config.mcp_events.contains(&"FolderStateChanged".to_string()));
        assert!(config.mcp_events.contains(&"DeviceConnected".to_string()));
        assert!(config.mcp_events.contains(&"DeviceDisconnected".to_string()));
        assert!(config.mcp_events.contains(&"LocalIndexUpdated".to_string()));
    }
}
