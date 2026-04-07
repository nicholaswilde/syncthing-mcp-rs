use serde_json::json;
use syncthing_mcp_rs::api::models::Config;
use syncthing_mcp_rs::tools::git_sync::ConfigExporter;

fn main() {
    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: syncthing_mcp_rs::api::models::GuiConfig {
            enabled: true,
            user: Some("admin".to_string()),
            password: Some("secret_password".to_string()),
            api_key: Some("very_secret_api_key".to_string()),
            ..Default::default()
        },
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

    println!("--- JSON Export ---");
    println!("{}", exporter.to_json().expect("Failed to export to JSON"));

    println!("\n--- YAML Export ---");
    println!("{}", exporter.to_yaml().expect("Failed to export to YAML"));
}
