use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::{Error, Result};
use serde_json::{Value, json};

/// Handler for the get_file_info tool.
pub async fn get_file_info(
    client: SyncThingClient,
    _config: AppConfig,
    params: Value,
) -> Result<Value> {
    let folder_id = params["folder_id"]
        .as_str()
        .ok_or_else(|| Error::ValidationError("folder_id is required".to_string()))?;
    let file_path = params["file_path"]
        .as_str()
        .ok_or_else(|| Error::ValidationError("file_path is required".to_string()))?;

    let info = client.get_file_info(folder_id, file_path).await?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": format!("File Info for '{}' in folder '{}':\nState: {}\nSize: {} bytes\nModified: {} (by {})\nAvailability: {} devices",
                    file_path,
                    folder_id,
                    info.global.file_type,
                    info.global.size,
                    info.mtime.value.real,
                    info.global.modified_by,
                    info.availability.len()
                )
            }
        ],
        "data": info
    }))
}

/// Handler for the get_folder_needs tool.
pub async fn get_folder_needs(
    client: SyncThingClient,
    _config: AppConfig,
    params: Value,
) -> Result<Value> {
    let folder_id = params["folder_id"]
        .as_str()
        .ok_or_else(|| Error::ValidationError("folder_id is required".to_string()))?;

    let page = params["page"].as_u64().map(|v| v as u32);
    let per_page = params["per_page"].as_u64().map(|v| v as u32);

    let needs = client.get_folder_needs(folder_id, page, per_page).await?;

    let mut summary = format!("Folder Needs: {}\n", folder_id);
    summary.push_str(&format!("Total items: {:?}\n", needs.total));
    summary.push_str(&format!("Progress: {} items\n", needs.progress.len()));
    summary.push_str(&format!("Queued: {} items\n", needs.queued.len()));
    summary.push_str(&format!("Rest: {} items\n", needs.rest.len()));

    if !needs.rest.is_empty() {
        summary.push_str("\nRemaining items (first 10):\n");
        for file in needs.rest.iter().take(10) {
            summary.push_str(&format!(
                "- {} ({} bytes, modified: {})\n",
                file.name, file.size, file.modified
            ));
        }
    }

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": summary
            }
        ],
        "data": needs
    }))
}

/// Handler for the get_discovery_status tool.
pub async fn get_discovery_status(
    client: SyncThingClient,
    _config: AppConfig,
    _params: Value,
) -> Result<Value> {
    let discovery = client.get_discovery_status().await?;

    let mut summary = format!("Discovery Status ({} devices):\n", discovery.len());

    if discovery.is_empty() {
        summary.push_str("No discovery information available.");
    } else {
        for (id, info) in discovery.iter().take(10) {
            summary.push_str(&format!("- {}: {:?}\n", id, info.addresses));
        }
        if discovery.len() > 10 {
            summary.push_str(&format!("... and {} more devices.", discovery.len() - 10));
        }
    }

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": summary
            }
        ],
        "data": discovery
    }))
}
