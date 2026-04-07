use std::env;
use syncthing_mcp_rs::api::SyncThingClient;
use syncthing_mcp_rs::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let run_live = std::env::var("RUN_LIVE_TESTS").unwrap_or_default();
    if run_live != "1" && run_live != "true" {
        println!("Skipping live test script (RUN_LIVE_TESTS not set to 1 or true)");
        return Ok(());
    }
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

    println!("--- Testing get_device_completion ---");
    // Use the first instance's own ID if possible, or just a placeholder if it fails
    let status = client.get_system_status().await?;
    let my_id = status.my_id;
    println!("Fetching completion for local device: {}", my_id);

    let completion = client.get_device_completion(&my_id, None).await?;
    println!("Completion: {:.2}%", completion.completion);
    println!("Global Bytes: {}", completion.global_bytes);
    println!("Need Bytes: {}", completion.need_bytes);
    println!("Global Items: {}", completion.global_items);
    println!("Need Items: {}", completion.need_items);

    println!("\n--- Testing set_file_priority ---");
    // We need a folder and a file that might be needed.
    // This is hard to guarantee in a live environment without setup,
    // so we'll try to find a folder and just call it with a dummy file
    // to verify the endpoint is reachable and responding with the expected model.
    let folders = client.list_folders().await?;
    if let Some(folder) = folders.first() {
        println!("Setting priority for 'dummy.txt' in folder: {}", folder.id);
        match client.set_file_priority(&folder.id, "dummy.txt").await {
            Ok(needs) => {
                println!("Successfully reached endpoint.");
                println!("Total items needed in folder: {:?}", needs.total);
            }
            Err(e) => {
                println!(
                    "Endpoint reached but returned error (expected if file not needed): {}",
                    e
                );
            }
        }
    } else {
        println!("No folders found to test set_file_priority.");
    }

    Ok(())
}
