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

/// Analyzes discovery and connection states to report network issues.
pub async fn diagnose_network_issues(
    client: SyncThingClient,
    _config: AppConfig,
    _params: Value,
) -> Result<Value> {
    let connections_resp = client.get_connections().await?;
    let discovery = client.get_discovery_status().await?;
    let status = client.get_system_status().await?;

    let my_id = status.my_id.clone();
    let mut text = String::from("Network Diagnostics Report\n");
    text.push_str("==========================\n\n");

    for (device_id, conn) in connections_resp.connections {
        if device_id == my_id {
            continue;
        }

        text.push_str(&format!("DEVICE: {}\n", device_id));

        if conn.connected {
            let conn_type = conn.connection_type.as_deref().unwrap_or("unknown");
            if conn_type.contains("relay") {
                text.push_str("  Status: Connected (Degraded via Relay)\n");
                text.push_str("  Recommendation: Check port forwarding (22000/TCP) and local firewalls to allow direct connections.\n");
            } else {
                text.push_str(&format!("  Status: Connected ({})\n", conn_type));
            }
        } else {
            text.push_str("  Status: Offline\n");
            if let Some(disco_info) = discovery.get(&device_id) {
                if disco_info.addresses.is_empty() {
                    text.push_str("  Discovery: Device is not announcing any addresses. It may be offline or completely disconnected from the discovery servers.\n");
                } else {
                    text.push_str("  Discovery: Found addresses, but cannot connect.\n");
                    for addr in &disco_info.addresses {
                        text.push_str(&format!("    - {}\n", addr));
                    }
                    text.push_str("  Recommendation: Device is likely blocked by a firewall or network routing issue. Verify it is reachable on these addresses.\n");
                }
            } else {
                text.push_str("  Discovery: No discovery information available. Device may be off or ignoring discovery.\n");
            }
        }
        text.push('\n');
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end()
        }]
    }))
}
