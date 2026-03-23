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
        _ => Err(crate::error::Error::Internal(format!(
            "Unsupported action: {}",
            action
        ))),
    }
}
