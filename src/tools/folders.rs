//! Folder management tools for SyncThing.

use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::{Value, json};

/// Manages SyncThing folders (list, add, remove, pause, resume).
pub async fn manage_folders(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let action = args["action"].as_str().unwrap_or("list");

    match action {
        "list" => {
            let folders = client.list_folders().await?;
            let mut text = String::from("SyncThing Folders:\n");
            for folder in folders {
                text.push_str(&format!(
                    "- {} ({}): {} (paused: {})\n",
                    folder.label, folder.id, folder.path, folder.paused
                ));
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "pending" => {
            let pending = client.get_pending_folders().await?;
            let mut text = String::from("Pending Folder Requests:\n");
            if pending.is_empty() {
                text.push_str("No pending folder requests.\n");
            } else {
                for (folder_id, folder) in pending {
                    text.push_str(&format!("- {} ({}):\n", folder_id, folder_id));
                    for (device_id, offered) in folder.offered_by {
                        text.push_str(&format!(
                            "  Offered by: {} (label: {}, time: {})\n",
                            device_id, offered.label, offered.time
                        ));
                    }
                }
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "reject_pending" => {
            let folder_id = args["folder_id"].as_str().ok_or_else(|| {
                crate::error::Error::Internal(
                    "folder_id is required for reject_pending".to_string(),
                )
            })?;
            client.remove_pending_folder(folder_id).await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Pending folder {} rejected successfully", folder_id)
                }]
            }))
        }
        "revert" => {
            let folder_id = args["folder_id"].as_str().ok_or_else(|| {
                crate::error::Error::Internal("folder_id is required for revert".to_string())
            })?;
            client.revert_folder(folder_id).await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Successfully triggered revert for folder: {}", folder_id)
                }]
            }))
        }
        _ => Err(crate::error::Error::Internal(format!(
            "Unsupported action: {}",
            action
        ))),
    }
}

/// Configures folder sharing between devices.
pub async fn configure_sharing(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let action = args["action"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("action is required".to_string()))?;
    let folder_id = args["folder_id"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("folder_id is required".to_string()))?;
    let device_id = args["device_id"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("device_id is required".to_string()))?;

    let mut folder = client.get_folder(folder_id).await?;

    match action {
        "share" => {
            if !folder.devices.iter().any(|d| d.device_id == device_id) {
                folder
                    .devices
                    .push(crate::api::models::FolderDeviceConfiguration {
                        device_id: device_id.to_string(),
                    });
                client
                    .patch_folder(folder_id, json!({"devices": folder.devices}))
                    .await?;
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Folder {} shared with device {} successfully", folder_id, device_id)
                }]
            }))
        }
        "unshare" => {
            let original_len = folder.devices.len();
            folder.devices.retain(|d| d.device_id != device_id);
            if folder.devices.len() != original_len {
                client
                    .patch_folder(folder_id, json!({"devices": folder.devices}))
                    .await?;
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Folder {} unshared from device {} successfully", folder_id, device_id)
                }]
            }))
        }
        _ => Err(crate::error::Error::Internal(format!(
            "Unsupported action: {}",
            action
        ))),
    }
}

/// Manages folder ignore patterns.
pub async fn manage_ignores(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let action = args["action"].as_str().unwrap_or("get");
    let folder_id = args["folder_id"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("folder_id is required".to_string()))?;

    match action {
        "get" => {
            let ignores = client.get_ignores(folder_id).await?;
            let mut text = format!("Ignore patterns for folder {}:\n", folder_id);
            let ignore_list = ignores.ignore.unwrap_or_default();
            if ignore_list.is_empty() {
                text.push_str("(No ignore patterns set)");
            } else {
                for pattern in &ignore_list {
                    text.push_str(&format!("- {}\n", pattern));
                }
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "set" => {
            let patterns = args["patterns"].as_array().ok_or_else(|| {
                crate::error::Error::Internal("patterns array is required for 'set'".to_string())
            })?;
            let patterns: Vec<String> = patterns
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            client.set_ignores(folder_id, patterns.clone()).await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Successfully set {} ignore patterns for folder {}", patterns.len(), folder_id)
                }]
            }))
        }
        "append" => {
            let new_patterns = args["patterns"].as_array().ok_or_else(|| {
                crate::error::Error::Internal("patterns array is required for 'append'".to_string())
            })?;
            let new_patterns: Vec<String> = new_patterns
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            let current_ignores = client.get_ignores(folder_id).await?;
            let mut ignore_list = current_ignores.ignore.unwrap_or_default();
            let mut added_count = 0;
            for pattern in new_patterns {
                if !ignore_list.contains(&pattern) {
                    ignore_list.push(pattern);
                    added_count += 1;
                }
            }

            if added_count > 0 {
                client.set_ignores(folder_id, ignore_list.clone()).await?;
            }

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Successfully appended {} new ignore patterns to folder {} (Total: {})", added_count, folder_id, ignore_list.len())
                }]
            }))
        }
        _ => Err(crate::error::Error::Internal(format!(
            "Unsupported action: {}",
            action
        ))),
    }
}

/// Retrieves statistics for all folders.
pub async fn get_folder_stats(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let stats = client.get_folder_stats().await?;

    let mut text = String::from("SyncThing Folder Statistics:\n\n");
    for (folder_id, folder_stats) in stats {
        text.push_str(&format!("Folder: {}\n", folder_id));
        text.push_str(&format!("  Last Scan: {}\n", folder_stats.last_scan));
        if let Some(last_file) = folder_stats.last_file {
            text.push_str("  Last Synced File:\n");
            text.push_str(&format!("    Filename: {}\n", last_file.filename));
            text.push_str(&format!("    At: {}\n\n", last_file.at));
        } else {
            text.push_str("  Last Synced File: None\n\n");
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end()
        }]
    }))
}
