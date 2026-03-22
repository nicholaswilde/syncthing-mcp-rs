use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::{json, Value};

pub async fn get_system_stats(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let status = client.get_system_status().await?;
    let version = client.get_system_version().await?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "SyncThing Version: {}\nUptime: {} seconds\nMemory Alloc: {} bytes\nMy ID: {}",
                version.version, status.uptime, status.alloc, status.my_id
            )
        }]
    }))
}
