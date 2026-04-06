//! Folder management tools for SyncThing.

use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::{Error, Result};
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
        "get" => {
            let folder_id = args["folder_id"].as_str().ok_or_else(|| {
                Error::Internal("folder_id is required for get action".to_string())
            })?;
            let folder = client.get_folder(folder_id).await?;
            Ok(json!({
                "content": [{
                    "type": "json",
                    "json": serde_json::to_value(folder)?
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

/// Provides a comprehensive status overview for a specific folder.
/// Consolidates sync status, conflicts, and statistics.
pub async fn inspect_folder(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let folder_id = args["folder_id"]
        .as_str()
        .ok_or_else(|| Error::ValidationError("folder_id is required".to_string()))?;

    // 1. Get Folder Config (to get path and label)
    let folder_config = client.get_folder(folder_id).await?;
    let path = std::path::Path::new(&folder_config.path);

    // 2. Get Folder Status (sync progress)
    let status = client.get_folder_status(folder_id).await?;
    let completion_pct = if status.global_bytes > 0 {
        (status.in_sync_bytes as f64 / status.global_bytes as f64) * 100.0
    } else {
        100.0
    };

    // 3. Get Folder Stats
    let all_stats = client.get_folder_stats().await?;
    let folder_stats = all_stats.get(folder_id);

    // 4. Scan for Conflicts
    let conflicts = crate::tools::conflicts::scan_conflicts(path)
        .await
        .unwrap_or_default();

    // 5. Get Device Completion (optional)
    let include_devices = args.get("include_devices").and_then(|v| v.as_bool()).unwrap_or(false);
    let mut device_completions = Vec::new();
    if include_devices {
        for device in &folder_config.devices {
            let completion = client.get_device_completion(&device.device_id, Some(folder_id)).await?;
            device_completions.push(json!({
                "device_id": device.device_id,
                "completion": completion
            }));
        }
    }

    // 6. Check if JSON output is requested
    if args.get("format").and_then(|v| v.as_str()) == Some("json") {
        let mut data = json!({
            "folder_id": folder_id,
            "label": folder_config.label,
            "status": status,
            "stats": folder_stats,
            "conflicts": conflicts
        });

        if include_devices {
            data["device_completions"] = json!(device_completions);
        }

        data = crate::mcp::optimization::optimize_response(data, &args);

        return Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&data).unwrap()
            }]
        }));
    }

    // 7. Build Combined Report (Text)
    let mut text = format!(
        "### Folder Overview: {} ({})\n\n",
        folder_config.label, folder_id
    );

    text.push_str("#### Sync Status\n");
    text.push_str(&format!("- **State**: {}\n", status.state));
    text.push_str(&format!("- **Completion**: {:.2}%\n", completion_pct));
    text.push_str(&format!(
        "- **Global Data**: {} bytes ({} files)\n",
        status.global_bytes, status.global_files
    ));
    text.push_str(&format!(
        "- **Local Data**: {} bytes\n",
        status.in_sync_bytes
    ));
    if status.need_bytes > 0 || status.need_files > 0 {
        text.push_str(&format!(
            "- **Syncing**: {} bytes remaining ({} files)\n",
            status.need_bytes, status.need_files
        ));
    }
    text.push('\n');

    if include_devices && !device_completions.is_empty() {
        text.push_str("#### Per-Device Completion\n");
        for dev_comp in device_completions {
            let device_id = dev_comp["device_id"].as_str().unwrap();
            let completion = dev_comp["completion"]["completion"].as_f64().unwrap();
            text.push_str(&format!("- **{}**: {:.2}%\n", device_id, completion));
        }
        text.push('\n');
    }

    text.push_str("#### Statistics\n");
    if let Some(stats) = folder_stats {
        text.push_str(&format!("- **Last Scan**: {}\n", stats.last_scan));
        if let Some(last_file) = &stats.last_file {
            text.push_str(&format!(
                "- **Last Synced File**: {} (at {})\n",
                last_file.filename, last_file.at
            ));
        }
    } else {
        text.push_str("- No statistics available.\n");
    }
    text.push('\n');

    text.push_str("#### Conflicts\n");
    if conflicts.is_empty() {
        text.push_str("- No conflicts found.\n");
    } else {
        text.push_str(&format!("- **{} conflict(s) found**:\n", conflicts.len()));
        for conflict in conflicts {
            let filename = std::path::Path::new(&conflict.conflict_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            text.push_str(&format!(
                "  - `{}` ({} bytes, from device {})\n",
                filename, conflict.conflict_size, conflict.device_id
            ));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end().to_string()
        }]
    }))
}

/// Performs bulk actions on multiple folders simultaneously.
pub async fn batch_manage_folders(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let folder_ids = args["folder_ids"].as_array().ok_or_else(|| {
        Error::ValidationError("folder_ids must be an array of strings".to_string())
    })?;

    let action = args["action"]
        .as_str()
        .ok_or_else(|| Error::ValidationError("action is required".to_string()))?;

    let mut results = Vec::new();
    let mut success_count = 0;

    for id_val in folder_ids {
        let id = id_val
            .as_str()
            .ok_or_else(|| Error::ValidationError("Each folder_id must be a string".to_string()))?;

        let res = match action {
            "rescan" => client.rescan(Some(id)).await,
            "revert" => client.revert_folder(id).await,
            "pause" => client
                .patch_folder(id, json!({"paused": true}))
                .await
                .map(|_| ()),
            "resume" => client
                .patch_folder(id, json!({"paused": false}))
                .await
                .map(|_| ()),
            _ => {
                return Err(Error::ValidationError(format!(
                    "Unsupported batch action: {}",
                    action
                )));
            }
        };

        match res {
            Ok(_) => {
                results.push(json!({"id": id, "status": "success"}));
                success_count += 1;
            }
            Err(e) => results.push(json!({"id": id, "status": "error", "message": e.to_string()})),
        }
    }

    // Check if JSON output is requested
    if args.get("format").and_then(|v| v.as_str()) == Some("json") {
        let mut data = json!({
            "action": action,
            "success_count": success_count,
            "results": results
        });

        data = crate::mcp::optimization::optimize_response(data, &args);

        return Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&data).unwrap()
            }]
        }));
    }

    let mut text = format!(
        "Successfully triggered {} for {} folder(s):\n\n",
        action, success_count
    );
    for res in results {
        let id = res["id"].as_str().unwrap();
        let status = res["status"].as_str().unwrap();
        if status == "success" {
            text.push_str(&format!("- {}: Success\n", id));
        } else {
            text.push_str(&format!(
                "- {}: Error - {}\n",
                id,
                res["message"].as_str().unwrap_or("Unknown error")
            ));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end().to_string()
        }]
    }))
}

/// Sets the synchronization priority for a specific file.
pub async fn set_file_priority(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let folder_id = args["folder_id"]
        .as_str()
        .ok_or_else(|| Error::ValidationError("folder_id is required".to_string()))?;
    let file_path = args["file_path"]
        .as_str()
        .ok_or_else(|| Error::ValidationError("file_path is required".to_string()))?;

    let needs = client.set_file_priority(folder_id, file_path).await?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Priority set successfully for '{}' in folder '{}'.", file_path, folder_id)
        }],
        "data": needs
    }))
}
