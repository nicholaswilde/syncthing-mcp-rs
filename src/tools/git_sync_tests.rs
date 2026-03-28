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
