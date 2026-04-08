//! Event timeline tools for SyncThing.

use crate::api::SyncThingClient;
use crate::api::models::EventData;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
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
    
    // Analyze patterns for intelligence
    let mut insights = Vec::new();
    let mut device_events: HashMap<String, Vec<&crate::api::models::Event>> = HashMap::new();
    
    for e in &events {
        if let Some(data) = &e.data {
            match data {
                EventData::DeviceConnected { device, .. } | 
                EventData::DeviceDisconnected { device, .. } => {
                    device_events.entry(device.clone()).or_default().push(e);
                }
                _ => {}
            }
        }
    }
    
    for (device, evs) in device_events {
        if evs.len() >= 3 {
            let mut connect_count = 0;
            let mut disconnect_count = 0;
            for e in &evs {
                if e.event_type == "DeviceConnected" { connect_count += 1; }
                if e.event_type == "DeviceDisconnected" { disconnect_count += 1; }
            }
            if connect_count >= 2 && disconnect_count >= 1 {
                insights.push(format!("Insight: Rapid flapping detected for device '{}'", device));
            }
        }
    }

    let summaries: Vec<_> = events.iter().map(|e| e.to_summary()).collect();
    
    let mut text = format!("Event Timeline (last {} seconds):\n", duration_s);
    
    if !insights.is_empty() {
        text.push_str("\nInsights:\n");
        for insight in insights {
            text.push_str(&format!("- {}\n", insight));
        }
        text.push_str("\nEvents:\n");
    }

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
