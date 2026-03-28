use serde_json::json;
use std::env;
use syncthing_mcp_rs::api::SyncThingClient;
use syncthing_mcp_rs::config::{AppConfig, BandwidthLimits, PerformanceProfile};
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
    
    // Add a test profile
    app_config.bandwidth.profiles.push(PerformanceProfile {
        name: "test_live_profile".to_string(),
        limits: BandwidthLimits {
            max_recv_kbps: Some(1234),
            max_send_kbps: Some(567),
        },
    });
    
    app_config.validate()?;

    let client = SyncThingClient::new(app_config.instances[0].clone());
    let registry = create_registry();

    println!("--- Testing get_bandwidth_status (Initial) ---");
    let tool = registry
        .get_tool("get_bandwidth_status")
        .expect("Tool get_bandwidth_status not found");
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing set_bandwidth_limits (Setting to 2000/1000) ---");
    let tool = registry
        .get_tool("set_bandwidth_limits")
        .expect("Tool set_bandwidth_limits not found");
    let result = (tool.handler)(
        &client,
        &app_config,
        Some(json!({
            "max_recv_kbps": 2000,
            "max_send_kbps": 1000
        })),
    )
    .await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing get_bandwidth_status (After explicit set) ---");
    let tool = registry
        .get_tool("get_bandwidth_status")
        .expect("Tool get_bandwidth_status not found");
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing set_performance_profile (Applying 'test_live_profile') ---");
    let tool = registry
        .get_tool("set_performance_profile")
        .expect("Tool set_performance_profile not found");
    let result = (tool.handler)(
        &client,
        &app_config,
        Some(json!({
            "name": "test_live_profile"
        })),
    )
    .await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Testing get_bandwidth_status (After profile application) ---");
    let tool = registry
        .get_tool("get_bandwidth_status")
        .expect("Tool get_bandwidth_status not found");
    let result = (tool.handler)(&client, &app_config, Some(json!({}))).await?;
    println!("{}", result["content"][0]["text"].as_str().unwrap());

    println!("\n--- Resetting to Unlimited (0/0) ---");
    let tool = registry
        .get_tool("set_bandwidth_limits")
        .expect("Tool set_bandwidth_limits not found");
    let _ = (tool.handler)(
        &client,
        &app_config,
        Some(json!({
            "max_recv_kbps": 0,
            "max_send_kbps": 0
        })),
    )
    .await?;
    println!("Reset successfully.");

    Ok(())
}
