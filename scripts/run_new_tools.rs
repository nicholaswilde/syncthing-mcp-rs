use syncthing_mcp_rs::api::SyncThingClient;
use syncthing_mcp_rs::config::AppConfig;
use syncthing_mcp_rs::tools::create_registry;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file manually if possible, or just expect vars to be set
    // For this script, we'll assume they are set via command line or exported
    
    let api_key = env::var("SYNCTHING_API_KEY").expect("SYNCTHING_API_KEY must be set");
    let host = env::var("SYNCTHING_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("SYNCTHING_PORT").unwrap_or_else(|_| "8384".to_string()).parse::<u16>()?;

    let mut app_config = AppConfig {
        host: host.clone(),
        port,
        api_key: Some(api_key.clone()),
        ..Default::default()
    };
    app_config.validate()?;

    let client = SyncThingClient::new(app_config.instances[0].clone());
    let registry = create_registry();

    println!("--- Testing get_system_connections ---");
    let tool = registry.get_tool("get_system_connections").unwrap();
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing get_system_log ---");
    let tool = registry.get_tool("get_system_log").unwrap();
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing get_device_statistics ---");
    let tool = registry.get_tool("get_device_statistics").unwrap();
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing get_folder_statistics ---");
    let tool = registry.get_tool("get_folder_statistics").unwrap();
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing manage_folders pending ---");
    let tool = registry.get_tool("manage_folders").unwrap();
    let result = (tool.handler)(&client, &app_config, Some(json!({"action": "pending"}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing manage_devices validate ---");
    let status = client.get_system_status().await?;
    let my_id = status.my_id;
    let tool = registry.get_tool("manage_devices").unwrap();
    let result = (tool.handler)(&client, &app_config, Some(json!({"action": "validate", "device_id": my_id}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    Ok(())
}
