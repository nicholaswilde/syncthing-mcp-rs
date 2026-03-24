use super::*;
use crate::test_utils::ENV_LOCK;

#[test]
fn test_load_defaults() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        std::env::remove_var("SYNCTHING_HOST");
        std::env::remove_var("SYNCTHING_PORT");
    }

    let config = match AppConfig::load(None, vec![]).unwrap() {
        ConfigResult::Config(c) => *c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 8384);
    assert_eq!(config.retry_max_attempts, 3);
    assert_eq!(config.retry_initial_backoff_ms, 100);
}

#[test]
fn test_cli_override() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let args = vec![
        "app".to_string(),
        "--host".to_string(),
        "cli.com".to_string(),
        "--port".to_string(),
        "4000".to_string(),
    ];
    let config = match AppConfig::load(None, args).unwrap() {
        ConfigResult::Config(c) => c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };
    assert_eq!(config.host, "cli.com");
    assert_eq!(config.port, 4000);
}

#[test]
fn test_env_override() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        std::env::set_var("SYNCTHING_HOST", "env.com");
        std::env::set_var("SYNCTHING_PORT", "5050");
    }

    let config = match AppConfig::load(None, vec![]).unwrap() {
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

#[test]
fn test_file_override() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use std::io::Write;
    let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        file,
        "host = \"file.com\"\nport = 6060\nretry_max_attempts = 5"
    )
    .unwrap();
    let path = file.path().to_str().unwrap().to_string();

    let config = match AppConfig::load(Some(path), vec![]).unwrap() {
        ConfigResult::Config(c) => *c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };
    assert_eq!(config.host, "file.com");
    assert_eq!(config.port, 6060);
    assert_eq!(config.retry_max_attempts, 5);
}

#[test]
fn test_multi_instance_loading() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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

    let config = match AppConfig::load(Some(path), vec![]).unwrap() {
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

#[test]
fn test_multi_instance_map_loading() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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

    let config = match AppConfig::load(Some(path), vec![]).unwrap() {
        ConfigResult::Config(c) => *c,
        ConfigResult::Exit => panic!("Expected Config, got Exit"),
    };
    assert_eq!(config.instances.len(), 2);
    // BTreeMap sorts by key, so inst1 should be first if visit_map uses it.
    // Actually, into_values() on BTreeMap will be in key order.
    assert!(config.instances.iter().any(|i| i.url == "http://inst1"));
    assert!(config.instances.iter().any(|i| i.url == "http://inst2"));
}

#[test]
fn test_config_validation_errors() {
    let mut config = AppConfig {
        instances: vec![InstanceConfig {
            url: "".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(config.validate().is_err());
    assert!(config.validate().unwrap_err().contains("missing URL"));

    config.instances = vec![];
    config.host = "".to_string();
    assert!(config.validate().is_err());
    assert!(
        config
            .validate()
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
