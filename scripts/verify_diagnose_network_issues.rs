use std::env;
use serde_json::json;
use syncthing_mcp_rs::api::client::SyncThingClient;
use syncthing_mcp_rs::config::AppConfig;
use syncthing_mcp_rs::config::InstanceConfig;
use syncthing_mcp_rs::tools::system::get_system_connections;
use syncthing_mcp_rs::tools::diagnostics::diagnose_network_issues;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("SYNCTHING_API_KEY").expect("SYNCTHING_API_KEY must be set");
    let url = env::var("SYNCTHING_URL").unwrap_or_else(|_| "http://localhost:8384".to_string());

    let config = InstanceConfig {
        url: url.clone(),
        api_key: Some(api_key),
        ..Default::default()
    };

    let client = SyncThingClient::new(config.clone());
    let app_config = AppConfig {
        instances: vec![config],
        ..Default::default()
    };

    println!("Testing get_system_connections in Analytics mode...");
    let args = json!({ "mode": "analytics" });
    match get_system_connections(client.clone(), app_config.clone(), args).await {
        Ok(res) => println!("{}", res["content"][0]["text"].as_str().unwrap_or("")),
        Err(e) => println!("Error: {}", e),
    }

    println!("\nTesting diagnose_network_issues...");
    let args_diag = json!({});
    match diagnose_network_issues(client, app_config, args_diag).await {
        Ok(res) => println!("{}", res["content"][0]["text"].as_str().unwrap_or("")),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
