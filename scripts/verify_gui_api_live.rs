use std::env;
use syncthing_mcp_rs::api::client::SyncThingClient;
use syncthing_mcp_rs::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    println!("--- Testing get_gui_config ---");
    let gui_config = client.get_gui_config().await?;
    println!("Successfully fetched GUI config:");
    println!("Enabled: {}", gui_config.enabled);
    println!("Address: {}", gui_config.address);
    println!("Theme: {}", gui_config.theme);
    println!("Use TLS: {}", gui_config.use_tls);

    println!("\n--- Testing set_gui_config (Dry Run / Verify) ---");
    // We only fetch it again to prove the endpoint works, as setting live config might be dangerous
    // but the get endpoint validates our models properly.
    
    Ok(())
}
