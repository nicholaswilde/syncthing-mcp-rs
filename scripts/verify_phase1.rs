use serde_json::json;
use std::env;
use syncthing_mcp_rs::api::client::SyncThingClient;
use syncthing_mcp_rs::config::InstanceConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let run_live = std::env::var("RUN_LIVE_TESTS").unwrap_or_default();
    if run_live != "1" && run_live != "true" {
        println!("Skipping live test script (RUN_LIVE_TESTS not set to 1 or true)");
        return Ok(());
    }
    let url = env::var("SYNCTHING_URL").unwrap_or_else(|_| "http://localhost:8384".to_string());
    let api_key = env::var("SYNCTHING_API_KEY").expect("SYNCTHING_API_KEY must be set");

    let config = InstanceConfig {
        url,
        api_key: Some(api_key),
        ..Default::default()
    };

    let client = SyncThingClient::new(config);

    // 1. Test patch_folder_config
    println!("--- Testing patch_folder_config ---");
    let folders = client.list_folders().await?;
    if let Some(folder) = folders.first() {
        let old_label = folder.label.clone();
        let new_label = format!("Verify-{}", chrono::Utc::now().timestamp());
        println!(
            "Patching folder {}: '{}' -> '{}'",
            folder.id, old_label, new_label
        );

        let patch = json!({ "label": new_label });
        let result = client.patch_folder_config(&folder.id, patch).await?;
        println!("Updated config: {}", serde_json::to_string_pretty(&result)?);

        let updated_label = result["label"].as_str().unwrap();
        if updated_label == new_label {
            println!("✅ patch_folder_config successful!");
        } else {
            println!(
                "❌ patch_folder_config failed: expected {}, got {}",
                new_label, updated_label
            );
        }
    } else {
        println!("⚠️ No folders found to test.");
    }

    // 2. Test patch_device_config
    println!("\n--- Testing patch_device_config ---");
    let devices = client.list_devices().await?;
    if let Some(device) = devices.first() {
        let old_name = device.name.clone().unwrap_or_default();
        let new_name = format!("Dev-{}", chrono::Utc::now().timestamp());
        println!(
            "Patching device {}: '{}' -> '{}'",
            device.device_id, old_name, new_name
        );

        let patch = json!({ "name": new_name });
        let result = client.patch_device_config(&device.device_id, patch).await?;
        println!("Updated config: {}", serde_json::to_string_pretty(&result)?);

        let updated_name = result["name"].as_str().unwrap_or_default();
        if updated_name == new_name {
            println!("✅ patch_device_config successful!");
        } else {
            println!(
                "❌ patch_device_config failed: expected {}, got {}",
                new_name, updated_name
            );
        }
    } else {
        println!("⚠️ No devices found to test.");
    }

    Ok(())
}
