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
    if let Some(v) = args.get("folders")
        && !v.is_array()
    {
        return Err(Error::ValidationError(
            "folders must be an array of strings".to_string(),
        ));
    }
    if let Some(v) = args.get("devices")
        && !v.is_array()
    {
        return Err(Error::ValidationError(
            "devices must be an array of strings".to_string(),
        ));
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
    let source_folders_all = source_config.folders.clone();
    let source_devices_all = source_config.devices.clone();

    // Validate that filtered IDs exist in source
    if let Some(filter) = folder_filter {
        let source_ids: HashSet<_> = source_folders_all.iter().map(|f| f.id.as_str()).collect();
        for id in filter {
            let id_str = id
                .as_str()
                .ok_or_else(|| Error::ValidationError("folder IDs must be strings".to_string()))?;
            if !source_ids.contains(id_str) {
                return Err(Error::ValidationError(format!(
                    "Folder not found in source: {}",
                    id_str
                )));
            }
        }
    }
    if let Some(filter) = device_filter {
        let source_ids: HashSet<_> = source_devices_all.iter().map(|d| d.device_id.as_str()).collect();
        for id in filter {
            let id_str = id
                .as_str()
                .ok_or_else(|| Error::ValidationError("device IDs must be strings".to_string()))?;
            if !source_ids.contains(id_str) {
                return Err(Error::ValidationError(format!(
                    "Device not found in source: {}",
                    id_str
                )));
            }
        }
    }

    // Filter folders if requested
    let source_folders = if let Some(filter) = folder_filter {
        let folder_ids: HashSet<_> = filter.iter().filter_map(|id| id.as_str()).collect();
        source_folders_all
            .iter()
            .filter(|f| folder_ids.contains(f.id.as_str()))
            .cloned()
            .collect()
    } else {
        source_folders_all.clone()
    };

    let mut source_devices = if let Some(filter) = device_filter {
        let device_ids: HashSet<_> = filter.iter().filter_map(|id| id.as_str()).collect();
        source_devices_all
            .iter()
            .filter(|d| device_ids.contains(d.device_id.as_str()))
            .cloned()
            .collect()
    } else {
        source_devices_all.clone()
    };

    // Add devices that are used by the filtered folders
    let mut required_device_ids = HashSet::new();
    for folder in &source_folders {
        for device in &folder.devices {
            required_device_ids.insert(device.device_id.clone());
        }
    }

    for device in &source_devices_all {
        if required_device_ids.contains(&device.device_id)
            && !source_devices
                .iter()
                .any(|d| d.device_id == device.device_id)
        {
            source_devices.push(device.clone());
        }
    }

    if folder_filter.is_some() && device_filter.is_none() {
        source_devices.retain(|d| required_device_ids.contains(&d.device_id));
    }

    // 5. Build difference report
    let mut source_config_filtered = source_config.clone();
    source_config_filtered.folders = source_folders.clone();
    source_config_filtered.devices = source_devices.clone();

    let diff =
        crate::tools::config_diff::ConfigDiff::generate(&source_config_filtered, &dest_config);
    let diff_summary = diff.summary();

    // 6. Update destination config
    dest_config.folders = source_folders;
    dest_config.devices = source_devices;

    // 7. Apply to destination
    if !dry_run {
        dest_client.set_config(dest_config).await?;
    }

    let status_prefix = if dry_run {
        format!(
            "[DRY RUN] Would replicate configuration to {}",
            destination_name
        )
    } else {
        format!(
            "Successfully replicated configuration to {}",
            destination_name
        )
    };

    let summary = format!("{}.\n{}", status_prefix, diff_summary);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": summary
        }]
    }))
}

/// Merges configuration from one SyncThing instance into another.
pub async fn merge_instance_configs(
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

    // 3. Fetch configs
    let source_config = source_client.get_config().await?;
    let mut dest_config = dest_client.get_config().await?;

    // 4. Generate diff and patch
    // We want source -> dest, so what's in source that is NOT in dest.
    let diff = crate::tools::config_diff::calculate_diff(&dest_config, &source_config);
    let patch = diff.to_patch();
    let summary = diff.summary();

    // 5. Apply patch
    if !dry_run {
        crate::tools::config_diff::apply_patch(&mut dest_config, &patch)?;
        dest_client.set_config(dest_config).await?;
    }

    let status_prefix = if dry_run {
        format!(
            "[DRY RUN] Would merge configuration from {} to {}",
            source_name.unwrap_or("default"),
            destination_name
        )
    } else {
        format!(
            "Successfully merged configuration from {} to {}",
            source_name.unwrap_or("default"),
            destination_name
        )
    };

    let summary = format!("{}.\n{}", status_prefix, summary);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": summary
        }]
    }))
}

/// Returns a diff between two SyncThing instance configurations.
pub async fn diff_instance_configs(
    client: SyncThingClient,
    config: AppConfig,
    args: Value,
) -> Result<Value> {
    let source_name = args.get("source").and_then(|v| v.as_str());
    let destination_name = args
        .get("destination")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::ValidationError("destination is required".to_string()))?;

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

    // 3. Fetch configs
    let source_config = source_client.get_config().await?;
    let dest_config = dest_client.get_config().await?;

    // 4. Generate diff
    let diff = crate::tools::config_diff::calculate_diff(&dest_config, &source_config);
    let summary = diff.summary();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": summary
        }]
    }))
}
