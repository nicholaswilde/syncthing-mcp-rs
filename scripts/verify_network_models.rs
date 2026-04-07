use std::env;
use syncthing_mcp_rs::api::client::SyncThingClient;
use syncthing_mcp_rs::config::InstanceConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("SYNCTHING_API_KEY").expect("SYNCTHING_API_KEY must be set");
    let url = env::var("SYNCTHING_URL").unwrap_or_else(|_| "http://localhost:8384".to_string());

    let config = InstanceConfig {
        url: url.clone(),
        api_key: Some(api_key),
        ..Default::default()
    };

    let client = SyncThingClient::new(config);

    println!("Testing get_connections on live instance...");
    match client.get_connections().await {
        Ok(response) => {
            println!("Successfully parsed ConnectionsResponse!");
            println!("Total IN: {} bytes, OUT: {} bytes", response.total.in_bytes_total, response.total.out_bytes_total);
            for (id, conn) in response.connections {
                println!("Device: {}", id);
                println!("  Connected: {}", conn.connected);
                println!("  Type: {:?}", conn.connection_type);
                println!("  Address: {:?}", conn.address);
                println!("  Crypto: {:?}", conn.crypto);
                println!("  Local: {:?}", conn.is_local);
            }
        }
        Err(e) => println!("Error getting connections: {}", e),
    }

    Ok(())
}
