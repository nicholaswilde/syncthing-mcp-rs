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

    println!("Testing get_file_info on 'default' folder...");
    // Try to find a file in the default folder to test with
    let folders = client.list_folders().await?;
    if let Some(folder) = folders.first() {
        let folder_id = &folder.id;
        println!("Using folder: {}", folder_id);
        
        // List some files first to find a valid one
        let browse = client.browse(folder_id, None, Some(1)).await?;
        if let Some(file_name) = browse.as_object().and_then(|o| o.keys().next()) {
            println!("Testing get_file_info on file: {}", file_name);
            let info = client.get_file_info(folder_id, file_name).await?;
            println!("File Info: {:?}", info.global.name);
        } else {
            println!("No files found in folder to test get_file_info");
        }

        println!("\nTesting get_folder_needs on folder: {}", folder_id);
        let needs = client.get_folder_needs(folder_id, None, None).await?;
        println!("Total needs: {:?}", needs.total);
    } else {
        println!("No folders found to test.");
    }

    println!("\nTesting get_discovery_status...");
    let discovery = client.get_discovery_status().await?;
    println!("Found discovery info for {} devices", discovery.len());
    if let Some((id, info)) = discovery.iter().next() {
        println!("Sample device: {} -> {:?}", id, info.addresses);
    }

    println!("\nTesting check_upgrade...");
    match client.check_upgrade().await {
        Ok(upgrade) => println!("Upgrade check: newer={}, latest={}", upgrade.newer, upgrade.latest),
        Err(e) => println!("Upgrade check failed: {}", e),
    }

    println!("\nTesting ping...");
    let ping = client.ping().await?;
    println!("Ping response: {}", ping.ping);

    Ok(())
}
