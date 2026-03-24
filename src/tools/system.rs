//! System management tools for SyncThing.

use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::{Error, Result};
use serde_json::{Value, json};

/// Retrieves system stats and version information from SyncThing.
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

/// Retrieves the synchronization status for a specific folder or device.
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
        _ => {
            return Err(Error::ValidationError(format!(
                "Invalid target: {}",
                target
            )));
        }
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}

/// Performs system maintenance tasks (rescan, restart, clear errors).
pub async fn maintain_system(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let action = args
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::ValidationError("Missing action argument".to_string()))?;

    match action {
        "force_rescan" => {
            let folder_id = args.get("folder_id").and_then(|v| v.as_str());
            client.rescan(folder_id).await?;
            let msg = if let Some(id) = folder_id {
                format!("Successfully triggered rescan for folder: {}", id)
            } else {
                "Successfully triggered rescan for all folders".to_string()
            };
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": msg
                }]
            }))
        }
        "restart" => {
            client.restart().await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": "Successfully triggered SyncThing restart"
                }]
            }))
        }
        "clear_errors" => {
            client.clear_errors().await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": "Successfully cleared SyncThing errors"
                }]
            }))
        }
        _ => Err(Error::ValidationError(format!(
            "Invalid action: {}",
            action
        ))),
    }
}

/// Retrieves the current connection status for all devices.
pub async fn get_system_connections(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let connections = client.get_connections().await?;

    let mut text = String::from("SyncThing Connection Status:\n\n");
    for (device_id, conn) in connections {
        text.push_str(&format!("Device: {}\n", device_id));
        text.push_str(&format!("  Connected: {}\n", conn.connected));
        if let Some(addr) = &conn.address {
            text.push_str(&format!("  Address: {}\n", addr));
        }
        if let Some(version) = &conn.client_version {
            text.push_str(&format!("  Version: {}\n", version));
        }
        if let Some(conn_type) = &conn.connection_type {
            text.push_str(&format!("  Type: {}\n", conn_type));
        }
        text.push_str(&format!("  In Bytes: {}\n", conn.in_bytes_total));
        text.push_str(&format!("  Out Bytes: {}\n", conn.out_bytes_total));
        text.push_str(&format!("  Paused: {}\n\n", conn.is_paused));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end()
        }]
    }))
}
