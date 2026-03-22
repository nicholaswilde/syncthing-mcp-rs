use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::{Error, Result};
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

pub async fn get_sync_status(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let target = args
        .get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::ValidationError("Missing target argument".to_string()))?;

    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::ValidationError("Missing id argument".to_string()))?;

    let text = match target {
        "folder" => {
            let status = client.get_folder_status(id).await?;
            let completion_pct = if status.global_bytes > 0 {
                (status.in_sync_bytes as f64 / status.global_bytes as f64) * 100.0
            } else {
                100.0
            };
            format!(
                "Folder: {}\nState: {}\nCompletion: {:.2}%\nBytes Remaining: {}\nFiles Remaining: {}",
                id, status.state, completion_pct, status.need_bytes, status.need_files
            )
        }
        "device" => {
            let completion = client.get_device_completion(id).await?;
            format!(
                "Device: {}\nCompletion: {:.2}%\nBytes Remaining: {}\nFiles Remaining: {}",
                id, completion.completion, completion.need_bytes, completion.need_files
            )
        }
        _ => return Err(Error::ValidationError(format!("Invalid target: {}", target))),
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}

