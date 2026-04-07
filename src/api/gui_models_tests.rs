use crate::api::models::GuiConfig;
use serde_json::json;

#[test]
fn test_gui_config_deserialization() {
    let json_data = json!({
        "enabled": true,
        "address": "127.0.0.1:8384",
        "user": "admin",
        "password": "hashed_password",
        "useTLS": true,
        "apiKey": "some-api-key",
        "theme": "dark",
        "debugging": false,
        "insecureAdminAccess": false,
        "insecureSkipHostcheck": false,
        "insecureAllowFrameAuth": false
    });

    let config: GuiConfig = serde_json::from_value(json_data).unwrap();

    assert!(config.enabled);
    assert_eq!(config.address, "127.0.0.1:8384");
    assert_eq!(config.user.as_deref(), Some("admin"));
    assert_eq!(config.password.as_deref(), Some("hashed_password"));
    assert!(config.use_tls);
    assert_eq!(config.api_key.as_deref(), Some("some-api-key"));
    assert_eq!(config.theme, "dark");
    assert!(!config.insecure_admin_access);
}

#[test]
fn test_gui_config_default() {
    let config = GuiConfig::default();

    assert!(!config.enabled);
    assert_eq!(config.address, "");
    assert_eq!(config.theme, "");
    assert!(!config.use_tls);
}
