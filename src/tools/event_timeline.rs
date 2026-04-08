//! Event timeline tools for SyncThing.

use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::{Value, json};
use std::time::Duration;

/// Retrieves a timeline of events from SyncThing.
pub async fn get_event_timeline(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let duration_s = args
        .get("duration_s")
        .and_then(|v| v.as_u64())
        .unwrap_or(3600); // Default to 1 hour

    let events = client.get_events_since_duration(Duration::from_secs(duration_s)).await?;
    
    let summaries: Vec<_> = events.iter().map(|e| e.to_summary()).collect();
    
    let mut text = format!("Event Timeline (last {} seconds):\n", duration_s);
    if summaries.is_empty() {
        text.push_str("No events found.");
    } else {
        for s in summaries {
            text.push_str(&format!("[{}] {}: {}\n", s.time, s.event_type, s.summary));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}
