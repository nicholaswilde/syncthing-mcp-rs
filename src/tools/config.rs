use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::{Error, Result};
use serde_json::{Value, json};
use std::collections::HashSet;

/// Replicates the configuration from one SyncThing instance to another.
pub async fn replicate_config(
    client: SyncThingClient,
    config: AppConfig,
    args: Value,
) -> Result<Value> {
    let source_name = args.get("source").and_then(|v| v.as_str());
    let destination_name = args
        .get("destination")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::ValidationError("destination is required".to_string()))?;
    let dry_run = args
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let folder_filter = args.get("folders").and_then(|v| v.as_array());
    let device_filter = args.get("devices").and_then(|v| v.as_array());

    // Validate filters are arrays if provided
    if let Some(v) = args.get("folders") {
        if !v.is_array() {
            return Err(Error::ValidationError(
                "folders must be an array of strings".to_string(),
            ));
        }
    }
    if let Some(v) = args.get("devices") {
        if !v.is_array() {
            return Err(Error::ValidationError(
                "devices must be an array of strings".to_string(),
            ));
        }
    }

    // 1. Get source client
    let source_client = if let Some(name) = source_name {
        let inst_config = config
            .get_instance(Some(name))
            .map_err(|e| Error::ValidationError(format!("Source instance not found: {}", e)))?;
        SyncThingClient::new(inst_config.clone())
    } else {
        client.clone()
    };

    // 2. Get destination client
    let dest_inst_config = config
        .get_instance(Some(destination_name))
        .map_err(|e| Error::ValidationError(format!("Destination instance not found: {}", e)))?;
    let dest_client = SyncThingClient::new(dest_inst_config.clone());

    // 3. Fetch source config
    let source_config = source_client.get_config().await?;
    let mut dest_config = dest_client.get_config().await?;

    // 4. Extract folders and devices from source
    let source_folders = source_config
        .get("folders")
        .and_then(|f| f.as_array())
        .cloned()
        .unwrap_or_default();
    let source_devices = source_config
        .get("devices")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    // Validate that filtered IDs exist in source
    if let Some(filter) = folder_filter {
        let source_ids: HashSet<_> = source_folders
            .iter()
            .filter_map(|f| f.get("id").and_then(|id| id.as_str()))
            .collect();
        for id in filter {
            let id_str = id.as_str().ok_or_else(|| {
                Error::ValidationError("folder IDs must be strings".to_string())
            })?;
            if !source_ids.contains(id_str) {
                return Err(Error::ValidationError(format!(
                    "Folder not found in source: {}",
                    id_str
                )));
            }
        }
    }
    if let Some(filter) = device_filter {
        let source_ids: HashSet<_> = source_devices
            .iter()
            .filter_map(|d| d.get("deviceID").and_then(|id| id.as_str()))
            .collect();
        for id in filter {
            let id_str = id.as_str().ok_or_else(|| {
                Error::ValidationError("device IDs must be strings".to_string())
            })?;
            if !source_ids.contains(id_str) {
                return Err(Error::ValidationError(format!(
                    "Device not found in source: {}",
                    id_str
                )));
            }
        }
    }

    // 5. Build difference report
    let mut diff_report = Vec::new();

    // Folders diff
    let empty_vec = Vec::new();
    let dest_folders = dest_config
        .get("folders")
        .and_then(|f| f.as_array())
        .unwrap_or(&empty_vec);
    let dest_folder_ids: HashSet<_> = dest_folders
        .iter()
        .filter_map(|f| f.get("id").and_then(|id| id.as_str()))
        .collect();
    let source_folder_ids: HashSet<_> = source_folders
        .iter()
        .filter_map(|f| f.get("id").and_then(|id| id.as_str()))
        .collect();

    let new_folders = source_folder_ids.difference(&dest_folder_ids).count();
    let removed_folders = dest_folder_ids.difference(&source_folder_ids).count();
    let shared_folders = source_folder_ids.intersection(&dest_folder_ids).count();

    diff_report.push(format!(
        "Folders: {} added, {} removed, {} updated.",
        new_folders, removed_folders, shared_folders
    ));

    // Devices diff
    let dest_devices = dest_config
        .get("devices")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty_vec);
    let dest_device_ids: HashSet<_> = dest_devices
        .iter()
        .filter_map(|d| d.get("deviceID").and_then(|id| id.as_str()))
        .collect();
    let source_device_ids: HashSet<_> = source_devices
        .iter()
        .filter_map(|d| d.get("deviceID").and_then(|id| id.as_str()))
        .collect();

    let new_devices = source_device_ids.difference(&dest_device_ids).count();
    let removed_devices = dest_device_ids.difference(&source_device_ids).count();
    let shared_devices = source_device_ids.intersection(&dest_device_ids).count();

    diff_report.push(format!(
        "Devices: {} added, {} removed, {} updated.",
        new_devices, removed_devices, shared_devices
    ));

    // 6. Update destination config
    if let Some(obj) = dest_config.as_object_mut() {
        obj.insert("folders".to_string(), Value::Array(source_folders.clone()));
        obj.insert("devices".to_string(), Value::Array(source_devices.clone()));
    }

    // 7. Apply to destination
    if !dry_run {
        dest_client.set_config(dest_config).await?;
    }

    let status_prefix = if dry_run {
        format!("[DRY RUN] Would replicate configuration to {}", destination_name)
    } else {
        format!("Successfully replicated configuration to {}", destination_name)
    };

    let summary = format!(
        "{}.\n{}",
        status_prefix,
        diff_report.join("\n")
    );

    Ok(json!({
        "content": [{
            "type": "text",
            "text": summary
        }]
    }))
}
