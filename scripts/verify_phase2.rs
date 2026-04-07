use syncthing_mcp_rs::api::client::SyncThingClient;
use syncthing_mcp_rs::config::{AppConfig, InstanceConfig};
use syncthing_mcp_rs::tools::config::patch_instance_config;
use std::env;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = env::var("SYNCTHING_URL").unwrap_or_else(|_| "http://localhost:8384".to_string());
    let api_key = env::var("SYNCTHING_API_KEY").expect("SYNCTHING_API_KEY must be set");

    let inst_config = InstanceConfig {
        name: Some("default".to_string()),
        url,
        api_key: Some(api_key),
        ..Default::default()
    };

    let client = SyncThingClient::new(inst_config.clone());
    let app_config = AppConfig {
        instances: vec![inst_config],
        ..Default::default()
    };

    // 1. Get folders to find one to patch
    let folders = client.list_folders().await?;
    if let Some(folder) = folders.first() {
        println!("--- Testing patch_instance_config (Dry Run) ---");
        let args_dry = json!({
            "folder_id": folder.id,
            "patch": { "label": "Dry-Run-Label" },
            "dry_run": true
        });
        let result_dry = patch_instance_config(client.clone(), app_config.clone(), args_dry).await?;
        println!("Dry run result:\n{}", result_dry["content"][0]["text"].as_str().unwrap());

        println!("\n--- Testing patch_instance_config (Apply) ---");
        let new_label = format!("Apply-{}", chrono::Utc::now().timestamp());
        let args_apply = json!({
            "folder_id": folder.id,
            "patch": { "label": new_label },
            "dry_run": false
        });
        let result_apply = patch_instance_config(client.clone(), app_config.clone(), args_apply).await?;
        let text = result_apply["content"][0]["text"].as_str().unwrap();
        println!("Apply result summary: {}", text.lines().next().unwrap());
        
        if text.contains(&new_label) {
            println!("✅ patch_instance_config successful!");
        } else {
            println!("❌ patch_instance_config failed to reflect new label in output.");
        }
    } else {
        println!("⚠️ No folders found to test.");
    }

    Ok(())
}
