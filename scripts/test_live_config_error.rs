use serde_json::json;
use std::env;
use syncthing_mcp_rs::api::SyncThingClient;
use syncthing_mcp_rs::config::AppConfig;
use syncthing_mcp_rs::tools::create_registry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    println!("--- Testing is_config_insync ---");
    let tool = registry
        .get_tool("is_config_insync")
        .expect("Tool is_config_insync not found");
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing get_system_errors ---");
    let tool = registry
        .get_tool("get_system_errors")
        .expect("Tool get_system_errors not found");
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing get_instance_overview (should now include config sync status) ---");
    let tool = registry
        .get_tool("get_instance_overview")
        .expect("Tool get_instance_overview not found");
    let result = (tool.handler)(&client, &app_config, Some(json!({"format": "text"}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing get_instance_health (should now include config sync status) ---");
    let tool = registry
        .get_tool("get_instance_health")
        .expect("Tool get_instance_health not found");
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    Ok(())
}
