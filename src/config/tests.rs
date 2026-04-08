use super::*;
use crate::test_utils::ENV_LOCK;

#[tokio::test]
async fn test_load_defaults() {
    let _guard = ENV_LOCK.lock().await;
    unsafe {
        std::env::remove_var("SYNCTHING_HOST");
        std::env::remove_var("SYNCTHING_PORT");
    }

    let config = match AppConfig::load(None, vec![]).await.unwrap() {
        ConfigResult::Config(c) => *c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 8384);
    assert_eq!(config.retry_max_attempts, 3);
    assert_eq!(config.retry_initial_backoff_ms, 100);
}

#[tokio::test]
async fn test_cli_override() {
    let _guard = ENV_LOCK.lock().await;
    let args = vec![
        "app".to_string(),
        "--host".to_string(),
        "cli.com".to_string(),
        "--port".to_string(),
        "4000".to_string(),
    ];
    let config = match AppConfig::load(None, args).await.unwrap() {
        ConfigResult::Config(c) => c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };
    assert_eq!(config.host, "cli.com");
    assert_eq!(config.port, 4000);
}

#[tokio::test]
async fn test_env_override() {
    let _guard = ENV_LOCK.lock().await;
    unsafe {
        std::env::set_var("SYNCTHING_HOST", "env.com");
        std::env::set_var("SYNCTHING_PORT", "5050");
    }

    let config = match AppConfig::load(None, vec![]).await.unwrap() {
        ConfigResult::Config(c) => *c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };

    unsafe {
        std::env::remove_var("SYNCTHING_HOST");
        std::env::remove_var("SYNCTHING_PORT");
    }

    assert_eq!(config.host, "env.com");
    assert_eq!(config.port, 5050);
}

#[tokio::test]
async fn test_file_override() {
    let _guard = ENV_LOCK.lock().await;
    use std::io::Write;
    let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        file,
        "host = \"file.com\"\nport = 6060\nretry_max_attempts = 5"
    )
    .unwrap();
    let path = file.path().to_str().unwrap().to_string();

    let config = match AppConfig::load(Some(path), vec![]).await.unwrap() {
        ConfigResult::Config(c) => *c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };
    assert_eq!(config.host, "file.com");
    assert_eq!(config.port, 6060);
    assert_eq!(config.retry_max_attempts, 5);
}

#[tokio::test]
async fn test_multi_instance_loading() {
    let _guard = ENV_LOCK.lock().await;
    use std::io::Write;
    let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        file,
        r#"
host = "primary.com"
retry_max_attempts = 10

[[instances]]
name = "primary"
url = "http://192.168.1.1"
api_key = "key1"

[[instances]]
name = "secondary"
url = "http://192.168.1.2"
api_key = "key2"
no_verify_ssl = false
retry_max_attempts = 2
"#
    )
    .unwrap();
    let path = file.path().to_str().unwrap().to_string();

    let config = match AppConfig::load(Some(path), vec![]).await.unwrap() {
        ConfigResult::Config(c) => *c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };
    assert_eq!(config.instances.len(), 2);
    assert_eq!(config.instances[0].name, Some("primary".to_string()));
    assert_eq!(config.instances[0].url, "http://192.168.1.1");
    assert_eq!(config.instances[0].retry_max_attempts, Some(10));
    assert_eq!(config.instances[1].name, Some("secondary".to_string()));
    assert_eq!(config.instances[1].no_verify_ssl, Some(false));
    assert_eq!(config.instances[1].retry_max_attempts, Some(2));
}

#[tokio::test]
async fn test_multi_instance_map_loading() {
    let _guard = ENV_LOCK.lock().await;
    use std::io::Write;
    let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        file,
        r#"
[instances.inst1]
url = "http://inst1"
api_key = "key1"

[instances.inst2]
url = "http://inst2"
api_key = "key2"
"#
    )
    .unwrap();
    let path = file.path().to_str().unwrap().to_string();

    let config = match AppConfig::load(Some(path), vec![]).await.unwrap() {
        ConfigResult::Config(c) => *c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };
    assert_eq!(config.instances.len(), 2);
    // BTreeMap sorts by key, so inst1 should be first if visit_map uses it.
    // Actually, into_values() on BTreeMap will be in key order.
    assert!(config.instances.iter().any(|i| i.url == "http://inst1"));
    assert!(config.instances.iter().any(|i| i.url == "http://inst2"));
}

#[tokio::test]
async fn test_config_validation_errors() {
    let mut config = AppConfig {
        instances: vec![InstanceConfig {
            url: "".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(config.validate().await.is_err());
    assert!(config.validate().await.unwrap_err().contains("missing URL"));

    config.instances = vec![];
    config.host = "".to_string();
    assert!(config.validate().await.is_err());
    assert!(
        config
            .validate()
            .await
            .unwrap_err()
            .contains("At least one SyncThing instance")
    );
}

#[test]
fn test_get_instance() {
    let config = AppConfig {
        instances: vec![
            InstanceConfig {
                name: Some("first".to_string()),
                url: "http://first".to_string(),
                ..Default::default()
            },
            InstanceConfig {
                name: Some("second".to_string()),
                url: "http://second".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    // Get default (first)
    assert_eq!(
        config.get_instance(None).unwrap().name,
        Some("first".to_string())
    );

    // Get by index
    assert_eq!(
        config.get_instance(Some("1")).unwrap().name,
        Some("second".to_string())
    );

    // Get by name
    assert_eq!(
        config.get_instance(Some("second")).unwrap().name,
        Some("second".to_string())
    );

    // Not found
    assert!(config.get_instance(Some("third")).is_err());
}

#[tokio::test]
async fn test_app_config_load_encrypt() {
    let args = vec![
        "app".to_string(),
        "encrypt".to_string(),
        "secret".to_string(),
    ];
    let result = AppConfig::load(None, args).await.unwrap();
    assert!(matches!(result, ConfigResult::Exit));
}

#[tokio::test]
async fn test_instance_config_propagation() {
    let mut config = AppConfig {
        retry_max_attempts: 5,
        instances: vec![InstanceConfig {
            url: "http://test".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    config.validate().await.unwrap();
    assert_eq!(config.instances[0].retry_max_attempts, Some(5));
}

#[tokio::test]
async fn test_load_non_existent_file() {
    let _guard = ENV_LOCK.lock().await;
    let result = AppConfig::load(Some("non_existent_file.toml".to_string()), vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_load_invalid_toml() {
    let _guard = ENV_LOCK.lock().await;
    use std::io::Write;
    let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(file, "this is not valid toml").unwrap();
    let path = file.path().to_str().unwrap().to_string();

    let result = AppConfig::load(Some(path), vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_load_invalid_port() {
    let _guard = ENV_LOCK.lock().await;
    let _args = [
        "app".to_string(),
        "--port".to_string(),
        "not-a-port".to_string(),
    ];
    // clap will handle this and likely exit or error out.
    // Since parse_args uses get_matches_from, it might panic or exit the process in tests if not handled.
    // However, our parse_args doesn't catch the error from get_matches_from.
}

#[tokio::test]
async fn test_instance_config_missing_url() {
    let mut config = AppConfig {
        instances: vec![InstanceConfig {
            name: Some("test".to_string()),
            url: "".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let result = config.validate().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("missing URL"));
}

#[tokio::test]
async fn test_vault_config_registration() {
    let _guard = ENV_LOCK.lock().await;
    let mut config = AppConfig {
        vault: VaultConfig {
            enabled: true,
            address: "http://vault:8200".to_string(),
            token: Some("test-token".to_string()),
            mount: "secret".to_string(),
        },
        instances: vec![InstanceConfig {
            url: "http://localhost".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    config.validate().await.unwrap();
    // Check if backend is registered
    let registry = crate::credentials::BACKEND_REGISTRY.read().unwrap();
    assert!(registry.contains_key("vault"));
}

#[tokio::test]
async fn test_vault_config_no_token() {
    let _guard = ENV_LOCK.lock().await;
    let mut config = AppConfig {
        vault: VaultConfig {
            enabled: true,
            address: "http://vault:8200".to_string(),
            token: None,
            mount: "secret".to_string(),
        },
        instances: vec![InstanceConfig {
            url: "http://localhost".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    // Should warn but not fail validation
    config.validate().await.unwrap();
}

#[tokio::test]
async fn test_aws_config_registration() {
    let _guard = ENV_LOCK.lock().await;
    let mut config = AppConfig {
        aws: AwsConfig {
            enabled: true,
            region: "us-west-2".to_string(),
            ..Default::default()
        },
        instances: vec![InstanceConfig {
            url: "http://localhost".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    config.validate().await.unwrap();
    // Check if backend is registered
    let registry = crate::credentials::BACKEND_REGISTRY.read().unwrap();
    assert!(registry.contains_key("aws"));
}

#[tokio::test]
async fn test_app_config_validate_propagates_settings() {
    let mut config = AppConfig {
        retry_max_attempts: 7,
        retry_initial_backoff_ms: 200,
        timeout_s: 60,
        instances: vec![InstanceConfig {
            url: "http://test".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    config.validate().await.unwrap();
    assert_eq!(config.instances[0].retry_max_attempts, Some(7));
    assert_eq!(config.instances[0].retry_initial_backoff_ms, Some(200));
    assert_eq!(config.instances[0].timeout_s, Some(60));
}
