use std::env;
use std::time::Duration;
use syncthing_mcp_rs::api::client::SyncThingClient;
use syncthing_mcp_rs::config::InstanceConfig;

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

    // 1. Test get_events_since_duration
    println!("--- Testing get_events_since_duration (last 1 hour) ---");
    let events = client
        .get_events_since_duration(Duration::from_secs(3600))
        .await?;
    println!("Found {} events in the last hour.", events.len());

    if !events.is_empty() {
        println!("✅ get_events_since_duration successful!");

        // 2. Test to_summary
        println!("\n--- Testing to_summary on the first event ---");
        let first_event = &events[0];
        let summary = first_event.to_summary();
        println!("Original Event ID: {}", first_event.id);
        println!("Summary ID: {}", summary.id);
        println!("Summary Text: {}", summary.summary);

        if first_event.id == summary.id && !summary.summary.is_empty() {
            println!("✅ to_summary successful!");
        } else {
            println!("❌ to_summary failed!");
        }
    } else {
        println!("⚠️ No events found in the last hour. Try triggering some activity in SyncThing.");
    }

    Ok(())
}
