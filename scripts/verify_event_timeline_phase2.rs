use std::env;
use serde_json::json;
use syncthing_mcp_rs::api::client::SyncThingClient;
use syncthing_mcp_rs::config::{AppConfig, InstanceConfig};
use syncthing_mcp_rs::tools::event_timeline::get_event_timeline;

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
    let app_config = AppConfig::default();

    // 1. Test get_event_timeline tool
    println!("--- Testing get_event_timeline tool (last 1 hour) ---");
    let args = json!({ "duration_s": 3600 });
    let result = get_event_timeline(client, app_config, args).await?;
    
    let text = result["content"][0]["text"].as_str().unwrap();
    println!("Tool Output:\n{}", text);
    
    if text.contains("Event Timeline") {
        println!("✅ get_event_timeline tool successful!");
    } else {
        println!("❌ get_event_timeline tool failed!");
    }

    Ok(())
}
