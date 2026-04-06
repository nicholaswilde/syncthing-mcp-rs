use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::{Value, json};

/// Manages SyncThing devices (list, add, remove, pause, resume, discover, approve).
pub async fn manage_devices(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let action = args["action"].as_str().unwrap_or("list");

    match action {
        "list" => {
            let devices = client.list_devices().await?;
            let mut text = String::from("SyncThing Devices:\n");
            for device in devices {
                text.push_str(&format!(
                    "- {} ({}): (paused: {})\n",
                    device.name.as_deref().unwrap_or("unnamed"),
                    device.device_id,
                    device.paused
                ));
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "discover" => {
            let pending = client.get_pending_devices().await?;
            let mut text = String::from("Pending Device Requests:\n");
            if pending.is_empty() {
                text.push_str("No pending device requests.\n");
            } else {
                for (device_id, device) in pending {
                    text.push_str(&format!(
                        "- {} ({}): (address: {}, time: {})\n",
                        device.name, device_id, device.address, device.time
                    ));
                }
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "approve" => {
            let device_id = args["device_id"].as_str().ok_or_else(|| {
                crate::error::Error::Internal("device_id is required for approve".to_string())
            })?;
            let name = args["name"].as_str();

            // 1. Add device to config
            client.add_device(device_id, name).await?;

            // 2. Remove from pending
            let _ = client.remove_pending_device(device_id).await;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Device {} approved and added successfully", device_id)
                }]
            }))
        }
        "add" => {
            let device_id = args["device_id"].as_str().ok_or_else(|| {
                crate::error::Error::Internal("device_id is required".to_string())
            })?;
            let name = args["name"].as_str();
            client.add_device(device_id, name).await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Device {} added successfully", device_id)
                }]
            }))
        }
        "remove" => {
            let device_id = args["device_id"].as_str().ok_or_else(|| {
                crate::error::Error::Internal("device_id is required".to_string())
            })?;
            client.remove_device(device_id).await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Device {} removed successfully", device_id)
                }]
            }))
        }
        "pause" => {
            let device_id = args["device_id"].as_str().ok_or_else(|| {
                crate::error::Error::Internal("device_id is required".to_string())
            })?;
            client
                .patch_device(device_id, json!({"paused": true}))
                .await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Device {} paused successfully", device_id)
                }]
            }))
        }
        "resume" => {
            let device_id = args["device_id"].as_str().ok_or_else(|| {
                crate::error::Error::Internal("device_id is required".to_string())
            })?;
            client
                .patch_device(device_id, json!({"paused": false}))
                .await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Device {} resumed successfully", device_id)
                }]
            }))
        }
        "validate" => {
            let device_id = args["device_id"].as_str().ok_or_else(|| {
                crate::error::Error::Internal("device_id is required for validate".to_string())
            })?;
            let resp = client.validate_device_id(device_id).await?;
            if let Some(id) = resp.id {
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Device ID is valid. Canonical format: {}", id)
                    }]
                }))
            } else {
                let error = resp.error.unwrap_or_else(|| "Unknown error".to_string());
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Device ID is invalid: {}", error)
                    }]
                }))
            }
        }
        _ => Err(crate::error::Error::Internal(format!(
            "Unsupported action: {}",
            action
        ))),
    }
}

/// Retrieves statistics for all devices.
pub async fn get_device_stats(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let stats = client.get_device_stats().await?;

    let mut text = String::from("SyncThing Device Statistics:\n\n");
    for (device_id, device_stats) in stats {
        text.push_str(&format!("Device: {}\n", device_id));
        text.push_str(&format!("  Last Seen: {}\n", device_stats.last_seen));
        text.push_str(&format!(
            "  Last Connection Duration: {:.2}s\n\n",
            device_stats.last_connection_duration_s
        ));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end()
        }]
    }))
}

/// Provides a comprehensive status overview for a specific device.
/// Consolidates device completion status and statistics.
pub async fn inspect_device(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let device_id = args["device_id"]
        .as_str()
        .ok_or_else(|| crate::error::Error::ValidationError("device_id is required".to_string()))?;

    // 1. Get Device Config (to get name)
    let config = client.get_config().await?;
    let device_config = config.devices.iter().find(|d| d.device_id == device_id);
    let device_name = device_config
        .and_then(|d| d.name.as_deref())
        .unwrap_or("Unknown");

    // 2. Get Device Completion
    let completion = client.get_device_completion(device_id, None).await?;

    // 3. Get Device Stats
    let all_stats = client.get_device_stats().await?;
    let device_stats = all_stats.get(device_id);

    // 4. Check if JSON output is requested
    if args.get("format").and_then(|v| v.as_str()) == Some("json") {
        let mut data = json!({
            "device_id": device_id,
            "name": device_name,
            "completion": completion,
            "stats": device_stats
        });

        data = crate::mcp::optimization::optimize_response(data, &args);

        return Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&data).unwrap()
            }]
        }));
    }

    // 5. Build Combined Report (Text)
    let mut text = format!("### Device Overview: {} ({})\n\n", device_name, device_id);

    text.push_str("#### Sync Status\n");
    text.push_str(&format!(
        "- **Completion**: {:.2}%\n",
        completion.completion
    ));
    text.push_str(&format!(
        "- **Global Data**: {} bytes\n",
        completion.global_bytes
    ));
    if completion.need_bytes > 0 {
        text.push_str(&format!(
            "- **Syncing**: {} bytes remaining ({} files)\n",
            completion.need_bytes, completion.need_items
        ));
    }
    text.push('\n');

    text.push_str("#### Statistics\n");
    if let Some(stats) = device_stats {
        text.push_str(&format!("- **Last Seen**: {}\n", stats.last_seen));
        text.push_str(&format!(
            "- **Last Connection Duration**: {:.2}s\n",
            stats.last_connection_duration_s
        ));
    } else {
        text.push_str("- No statistics available.\n");
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end().to_string()
        }]
    }))
}
