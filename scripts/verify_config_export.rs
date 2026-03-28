use serde_json::json;
use syncthing_mcp_rs::api::models::Config;
use syncthing_mcp_rs::tools::git_sync::ConfigExporter;

fn main() {
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

    println!("--- JSON Export ---");
    println!("{}", exporter.to_json().expect("Failed to export to JSON"));

    println!("\n--- YAML Export ---");
    println!("{}", exporter.to_yaml().expect("Failed to export to YAML"));
}
