use std::env;
use syncthing_mcp_rs::api::SyncThingClient;
use syncthing_mcp_rs::config::InstanceConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("SYNCTHING_API_KEY").expect("SYNCTHING_API_KEY must be set");
    let url = env::var("SYNCTHING_URL").unwrap_or_else(|_| "http://localhost:8384".to_string());

    let config = InstanceConfig {
        url,
        api_key: Some(api_key),
        ..Default::default()
    };

    let client = SyncThingClient::new(config);

    println!("Performing health check...");
    let health = client.health_check().await?;

    println!("Status: {}", health.status);
    println!("Latency: {}ms", health.latency_ms);
    if let Some(version) = health.version {
        println!("Version: {}", version);
    }
    if let Some(error) = health.error {
        println!("Error: {}", error);
    }

    Ok(())
}
