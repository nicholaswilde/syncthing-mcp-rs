use serde_json::json;
use std::env;
use syncthing_mcp_rs::api::SyncThingClient;
use syncthing_mcp_rs::config::AppConfig;
use syncthing_mcp_rs::tools::create_registry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let run_live = std::env::var("RUN_LIVE_TESTS").unwrap_or_default();
    if run_live != "1" && run_live != "true" {
        println!("Skipping live test script (RUN_LIVE_TESTS not set to 1 or true)");
        return Ok(());
    }
    // Note: We assume the caller has sourced the .env file or is using `task` which does it.
    let api_key = env::var("SYNCTHING_API_KEY").expect("SYNCTHING_API_KEY must be set");
    let host = env::var("SYNCTHING_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("SYNCTHING_PORT")
        .unwrap_or_else(|_| "8384".to_string())
        .parse::<u16>()?;

    let mut app_config = AppConfig {
        host: host.clone(),
        port,
        api_key: Some(api_key.clone()),
        ..Default::default()
    };
    app_config.validate().await?;

    let client = SyncThingClient::new(app_config.instances[0].clone());
    let registry = create_registry();

    println!("--- Testing get_instance_overview ---");
    let tool = registry
        .get_tool("get_instance_overview")
        .expect("Tool get_instance_overview not found");
    let result = (tool.handler)(
        client.clone(),
        app_config.clone(),
        Some(json!({"format": "text"})),
    )
    .await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing summarize_conflicts ---");
    let tool = registry
        .get_tool("summarize_conflicts")
        .expect("Tool summarize_conflicts not found");
    let result = (tool.handler)(
        client.clone(),
        app_config.clone(),
        Some(json!({"format": "text"})),
    )
    .await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    let config = client.get_config().await?;
    if let Some(folder) = config.folders.first() {
        let folder_id = &folder.id;
        println!("\n--- Testing inspect_folder on {} ---", folder_id);
        let tool = registry
            .get_tool("inspect_folder")
            .expect("Tool inspect_folder not found");
        let result = (tool.handler)(
            client.clone(),
            app_config.clone(),
            Some(json!({"folder_id": folder_id, "format": "text"})),
        )
        .await?;
        println!("{}", result["content"][0]["text"].as_str().unwrap());

        println!(
            "\n--- Testing batch_manage_folders (pause) on {} ---",
            folder_id
        );
        let tool = registry
            .get_tool("batch_manage_folders")
            .expect("Tool batch_manage_folders not found");
        let result = (tool.handler)(
            client.clone(),
            app_config.clone(),
            Some(json!({
                "folder_ids": [folder_id],
                "action": "pause",
                "format": "text"
            })),
        )
        .await?;
        println!("{}", result["content"][0]["text"].as_str().unwrap());

        println!(
            "\n--- Testing batch_manage_folders (resume) on {} ---",
            folder_id
        );
        let result = (tool.handler)(
            client.clone(),
            app_config.clone(),
            Some(json!({
                "folder_ids": [folder_id],
                "action": "resume",
                "format": "text"
            })),
        )
        .await?;
        println!("{}", result["content"][0]["text"].as_str().unwrap());
    }

    if let Some(device) = config.devices.first() {
        let device_id = &device.device_id;
        println!("\n--- Testing inspect_device on {} ---", device_id);
        let tool = registry
            .get_tool("inspect_device")
            .expect("Tool inspect_device not found");
        let result = (tool.handler)(
            client.clone(),
            app_config.clone(),
            Some(json!({"device_id": device_id, "format": "text"})),
        )
        .await?;
        println!("{}", result["content"][0]["text"].as_str().unwrap());
    }

    Ok(())
}
