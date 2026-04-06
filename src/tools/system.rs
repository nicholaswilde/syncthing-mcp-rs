//! System management tools for SyncThing.

use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::{Error, Language, Result};
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
            let completion = client.get_device_completion(id, None).await?;
            format!(
                "Device: {}\nCompletion: {:.2}%\nBytes Remaining: {}\nFiles Remaining: {}",
                id, completion.completion, completion.need_bytes, completion.need_items
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
        "shutdown" => {
            client.shutdown().await?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": "Successfully triggered SyncThing shutdown"
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
    let response = client.get_connections().await?;

    let mut text = String::from("SyncThing Connection Status:\n\n");
    for (device_id, conn) in response.connections {
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
        text.push_str(&format!("  Paused: {}\n\n", conn.paused));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end()
        }]
    }))
}

/// Retrieves the recent system log entries from SyncThing.
pub async fn get_system_log(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let log = client.get_system_log().await?;

    let mut text = String::from("SyncThing System Log:\n\n");
    for entry in log.messages {
        text.push_str(&format!("[{}] {}\n", entry.when, entry.message));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end()
        }]
    }))
}

/// Lists all configured SyncThing instances and their current health status.
pub async fn list_instances(
    _client: SyncThingClient,
    config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let mut text = String::from("### SyncThing Instances Status\n\n");

    for (i, instance_config) in config.instances.iter().enumerate() {
        let name = instance_config
            .name
            .clone()
            .unwrap_or_else(|| format!("Instance {}", i));
        let client = SyncThingClient::new(instance_config.clone());
        let health = client.health_check().await?;

        let status_badge = match health.status.as_str() {
            "Online" => "🟢 Online",
            "Offline" => "🔴 Offline",
            _ => "🟡 Warning",
        };

        text.push_str(&format!("**Instance: {}**\n", name));
        text.push_str(&format!("- **URL**: {}\n", instance_config.url));
        text.push_str(&format!("- **Status**: {}\n", status_badge));
        if let Some(version) = health.version {
            text.push_str(&format!("- **Version**: {}\n", version));
        }
        text.push_str(&format!("- **Latency**: {}ms\n", health.latency_ms));
        if let Some(uptime) = health.uptime {
            text.push_str(&format!("- **Uptime**: {}s\n", uptime));
        }
        if let Some(alloc) = health.memory_alloc {
            text.push_str(&format!("- **Memory Alloc**: {} bytes\n", alloc));
        }
        if let Some(insync) = health.config_insync
            && !insync
        {
            text.push_str("- **⚠️ Configuration NOT in sync with on-disk (restart required)**\n");
        }
        if let Some(error) = health.error {
            text.push_str(&format!("- **Error**: {}\n", error));
        }
        text.push_str("\n---\n\n");
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end().trim_end_matches("---").trim_end().to_string()
        }]
    }))
}

/// Retrieves detailed health information for a specific SyncThing instance.
pub async fn get_instance_health(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let health = client.health_check().await?;

    let status_badge = match health.status.as_str() {
        "Online" => "🟢 Online",
        "Offline" => "🔴 Offline",
        _ => "🟡 Warning",
    };

    let name = client
        .config
        .name
        .clone()
        .unwrap_or_else(|| client.config.url.clone());

    let mut text = format!("SyncThing Health: {}\n", name);
    text.push_str(&format!("Status: {}\n", status_badge));
    if let Some(version) = health.version {
        text.push_str(&format!("Version: {}\n", version));
    }
    text.push_str(&format!("Latency: {}ms\n", health.latency_ms));
    if let Some(uptime) = health.uptime {
        text.push_str(&format!("Uptime: {} seconds\n", uptime));
    }
    if let Some(alloc) = health.memory_alloc {
        text.push_str(&format!("Memory Alloc: {} bytes\n", alloc));
    }
    if let Some(sys) = health.memory_sys {
        text.push_str(&format!("Memory Total: {} bytes\n", sys));
    }
    if let Some(insync) = health.config_insync
        && !insync
    {
        text.push_str("⚠️ Configuration NOT in sync with on-disk (restart required)\n");
    }
    if let Some(error) = health.error {
        text.push_str(&format!("Error: {}\n", error));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}

/// Analyzes a technical error message and provides a diagnostic summary.
pub async fn analyze_error(
    _client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let error_message = args
        .get("error_message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::ValidationError("Missing error_message argument".to_string()))?;

    let tool_name = args.get("tool_name").and_then(|v| v.as_str());

    // Try to map simple error strings to Error variants for diagnosis.
    let error = if error_message.to_lowercase().contains("401")
        || error_message.to_lowercase().contains("unauthorized")
    {
        Error::Unauthorized(error_message.to_string())
    } else if error_message.to_lowercase().contains("403")
        || error_message.to_lowercase().contains("forbidden")
    {
        Error::Forbidden(error_message.to_string())
    } else if error_message.to_lowercase().contains("404")
        || error_message.to_lowercase().contains("not found")
    {
        if error_message.to_lowercase().contains("folder")
            || error_message.to_lowercase().contains("device")
        {
            Error::SyncThing(error_message.to_string())
        } else {
            Error::NotFound(error_message.to_string())
        }
    } else if error_message.to_lowercase().contains("refused")
        || error_message.to_lowercase().contains("timeout")
    {
        Error::Network(error_message.to_string())
    } else {
        Error::SyncThing(error_message.to_string())
    };

    let diagnostic = if let Some(ctx) = tool_name {
        error.diagnose_with_context(Language::English, Some(ctx))
    } else {
        error.diagnose()
    };

    let text = format!(
        "### Error Analysis Result\n\n- **Category**: {}\n- **Explanation**: {}\n- **Advice**: {}",
        diagnostic.category, diagnostic.explanation, diagnostic.advice
    );

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}

/// Provides a top-level health and status report for a SyncThing instance.
/// Consolidates system status, connections, and version information.
pub async fn get_instance_overview(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    // 1. Get System Status
    let status = client.get_system_status().await?;

    // 2. Get Connections
    let connections = client.get_connections().await?;
    let connected_count = connections
        .connections
        .values()
        .filter(|c| c.connected)
        .count();

    // 3. Get Version
    let version = client.get_system_version().await?;

    // 4. Get Config In Sync
    let config_insync = client.is_config_insync().await?;

    // 5. Check if JSON output is requested
    if args.get("format").and_then(|v| v.as_str()) == Some("json") {
        let mut data = json!({
            "status": status,
            "connections_summary": {
                "total": connections.connections.len(),
                "connected": connected_count
            },
            "version": version,
            "config_insync": config_insync.insync
        });

        data = crate::mcp::optimization::optimize_response(data, &args);

        return Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&data).unwrap()
            }]
        }));
    }

    // 6. Build Combined Report (Text)
    let mut text = format!("### Instance Overview: {}\n\n", status.my_id);

    text.push_str("#### System Status\n");
    text.push_str(&format!("- **Version**: {}\n", version.version));
    text.push_str(&format!("- **Uptime**: {}s\n", status.uptime));
    text.push_str(&format!("- **Memory Usage**: {} bytes\n", status.alloc));
    text.push_str(&format!("- **OS/Arch**: {}/{}\n", version.os, version.arch));
    if !config_insync.insync {
        text.push_str("- **⚠️ Configuration NOT in sync with on-disk (restart required)**\n");
    }
    text.push('\n');

    text.push_str("#### Connectivity\n");
    text.push_str(&format!(
        "- **Connected Peers**: {} / {}\n",
        connected_count,
        connections.connections.len()
    ));
    if connected_count > 0 {
        text.push_str("- **Active Connections**:\n");
        for (id, conn) in connections.connections.iter().filter(|(_, c)| c.connected) {
            text.push_str(&format!(
                "  - `{}` ({})\n",
                id,
                conn.connection_type.as_deref().unwrap_or("unknown")
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

/// Checks if a newer version of SyncThing is available.
pub async fn check_upgrade(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    match client.check_upgrade().await {
        Ok(upgrade) => {
            let mut text = format!(
                "SyncThing Upgrade Check:\nRunning Version: {}\nLatest Version: {}\nNewer Available: {}\n",
                upgrade.running, upgrade.latest, upgrade.newer
            );
            if upgrade.major_newer {
                text.push_str("⚠️ A major version update is available!\n");
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }],
                "data": upgrade
            }))
        }
        Err(e) => {
            if e.to_string().contains("upgrade unsupported") {
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": "Internal SyncThing upgrades are not supported on this instance (likely managed by Docker or a package manager)."
                    }]
                }))
            } else {
                Err(e)
            }
        }
    }
}

/// Triggers an upgrade to the latest version of SyncThing.
pub async fn perform_upgrade(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    client.perform_upgrade().await?;
    Ok(json!({
        "content": [{
            "type": "text",
            "text": "Successfully triggered SyncThing upgrade. The instance will restart shortly."
        }]
    }))
}

/// Pings the SyncThing instance to verify API responsiveness.
pub async fn ping_instance(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let start = std::time::Instant::now();
    let resp = client.ping().await?;
    let latency = start.elapsed().as_millis();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Ping response: {} (latency: {}ms)", resp.ping, latency)
        }],
        "data": {
            "ping": resp.ping,
            "latency_ms": latency
        }
    }))
}

/// Checks if the running configuration is in sync with the on-disk configuration.
pub async fn is_config_insync(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let status = client.is_config_insync().await?;

    let text = if status.insync {
        "Configuration is in sync with the on-disk configuration."
    } else {
        "⚠️ Configuration is NOT in sync with the on-disk configuration. A restart might be required to apply changes."
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }],
        "data": status
    }))
}

/// Retrieves the current list of active system GUI errors from SyncThing.
pub async fn get_system_errors(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let errors = client.get_errors().await?;

    let error_list = errors.errors.as_ref().filter(|v| !v.is_empty());

    if error_list.is_none() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "No active system GUI errors found."
            }]
        }));
    }

    let mut text = String::from("### SyncThing System Errors\n\n");
    for error in error_list.unwrap() {
        text.push_str(&format!("- **[{}]** {}\n", error.when, error.message));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end().to_string()
        }],
        "data": errors
    }))
}
