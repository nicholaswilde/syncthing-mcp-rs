use crate::api::models::Config;
use crate::tools::git_sync::ConfigExporter;
use serde_json::json;

#[test]
fn test_export_config_to_json() {
    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: json!({"enabled": true}),
        ldap: json!({}),
        options: json!({"listenAddresses": ["default"]}),
        remote_ignored_devices: json!([]),
        defaults: json!({}),
    };

    let exporter = ConfigExporter::new(config);
    let exported = exporter.to_json().expect("Failed to export to JSON");
    
    // Verify it's pretty-printed JSON
    assert!(exported.contains("  \"version\": 37"));
    assert!(exported.contains("\"gui\": {"));
}

#[test]
fn test_export_config_to_yaml() {
    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: json!({"enabled": true}),
        ldap: json!({}),
        options: json!({"listenAddresses": ["default"]}),
        remote_ignored_devices: json!([]),
        defaults: json!({}),
    };

    let exporter = ConfigExporter::new(config);
    let exported = exporter.to_yaml().expect("Failed to export to YAML");
    
    // Verify it's YAML
    assert!(exported.contains("version: 37"));
    assert!(exported.contains("gui:"));
    assert!(exported.contains("enabled: true"));
}

#[test]
fn test_mask_sensitive_info() {
    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: json!({
            "enabled": true,
            "user": "admin",
            "password": "secret_password",
            "apiKey": "very_secret_api_key"
        }),
        ldap: json!({
            "enabled": true,
            "password": "ldap_password"
        }),
        options: json!({}),
        remote_ignored_devices: json!([]),
        defaults: json!({}),
    };

    let mut exporter = ConfigExporter::new(config);
    exporter.mask_sensitive();
    let exported = exporter.to_json().expect("Failed to export to JSON");
    
    // Verify sensitive info is masked
    assert!(exported.contains("\"user\": \"********\""));
    assert!(exported.contains("\"password\": \"********\""));
    assert!(exported.contains("\"apiKey\": \"********\""));
    
    // Verify non-sensitive info is preserved
    assert!(exported.contains("\"enabled\": true"));
}
