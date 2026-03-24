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
    app_config.validate()?;

    let client = SyncThingClient::new(app_config.instances[0].clone());
    let registry = create_registry();

    println!("--- Testing list_instances ---");
    let tool = registry
        .get_tool("list_instances")
        .expect("Tool list_instances not found");
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing get_instance_health ---");
    let tool = registry
        .get_tool("get_instance_health")
        .expect("Tool get_instance_health not found");
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing replicate_config (DRY RUN) ---");
    // We replicate to ourselves as a safe dry run test
    let tool = registry
        .get_tool("replicate_config")
        .expect("Tool replicate_config not found");
    let result = (tool.handler)(
        &client,
        &app_config,
        Some(json!({
            "destination": "0", // First instance
            "dry_run": true
        })),
    )
    .await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    Ok(())
}
