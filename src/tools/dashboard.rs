//! Global dashboard tool for aggregating statistics across all SyncThing instances.

use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Global dashboard report structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalDashboard {
    /// Summary of all instances.
    pub summary: DashboardSummary,
    /// Detailed status of each instance.
    pub instances: Vec<InstanceStatus>,
    /// Recent alerts or errors across the network.
    pub alerts: Vec<DashboardAlert>,
}

/// High-level summary of the entire SyncThing network.
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardSummary {
    /// Total number of instances.
    pub total_instances: usize,
    /// Number of online instances.
    pub online_instances: usize,
    /// Aggregated download rate (bytes/s).
    pub total_download_rate: u64,
    /// Aggregated upload rate (bytes/s).
    pub total_upload_rate: u64,
    /// Total data in-sync (bytes).
    pub total_in_sync_bytes: u64,
    /// Total data remaining to sync (bytes).
    pub total_need_bytes: u64,
    /// Overall synchronization percentage (0-100).
    pub overall_completion: f64,
}

/// Status of a single SyncThing instance.
#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceStatus {
    /// Friendly name of the instance.
    pub name: String,
    /// Current health status (Online, Offline, Warning).
    pub status: String,
    /// SyncThing version.
    pub version: Option<String>,
    /// Latency to the instance (ms).
    pub latency_ms: u128,
    /// Uptime in seconds.
    pub uptime: Option<u64>,
    /// Memory allocation (bytes).
    pub memory_alloc: Option<u64>,
    /// Download rate for this instance (bytes/s).
    pub download_rate: u64,
    /// Upload rate for this instance (bytes/s).
    pub upload_rate: u64,
    /// Number of connected devices.
    pub connected_devices: usize,
    /// Number of folders in sync.
    pub in_sync_folders: usize,
    /// Total number of folders.
    pub total_folders: usize,
}

/// An alert or error event in the dashboard.
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardAlert {
    /// The instance name where the alert originated.
    pub instance: String,
    /// The alert message.
    pub message: String,
    /// Severity level (Info, Warning, Error).
    pub severity: String,
}

/// Aggregates data from all configured SyncThing instances to create a global dashboard.
pub async fn get_global_dashboard(
    _client: SyncThingClient,
    config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let mut instance_futures = Vec::new();

    for (i, instance_config) in config.instances.iter().enumerate() {
        let name = instance_config
            .name
            .clone()
            .unwrap_or_else(|| format!("Instance {}", i));
        let client = SyncThingClient::new(instance_config.clone());
        instance_futures.push(fetch_instance_dashboard_data(name, client));
    }

    let results = join_all(instance_futures).await;

    let mut instances = Vec::new();
    let mut alerts = Vec::new();
    let mut total_download_rate = 0;
    let mut total_upload_rate = 0;
    let total_in_sync_bytes = 0;
    let total_need_bytes = 0;
    let mut online_count = 0;

    for result in results {
        match result {
            Ok(data) => {
                if data.status == "Online" {
                    online_count += 1;
                }
                total_download_rate += data.download_rate;
                total_upload_rate += data.upload_rate;
                instances.push(data);
            }
            Err(e) => {
                alerts.push(DashboardAlert {
                    instance: "Unknown".to_string(),
                    message: format!("Failed to fetch data: {}", e),
                    severity: "Error".to_string(),
                });
            }
        }
    }

    let summary = DashboardSummary {
        total_instances: config.instances.len(),
        online_instances: online_count,
        total_download_rate,
        total_upload_rate,
        total_in_sync_bytes,
        total_need_bytes,
        overall_completion: calculate_overall_completion(total_in_sync_bytes, total_need_bytes),
    };

    let dashboard = GlobalDashboard {
        summary,
        instances,
        alerts,
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format_dashboard_text(&dashboard)
        }],
        "data": dashboard
    }))
}

async fn fetch_instance_dashboard_data(
    name: String,
    client: SyncThingClient,
) -> Result<InstanceStatus> {
    let health = client.health_check().await?;
    let connections = if health.status == "Online" {
        client.get_connections().await.ok()
    } else {
        None
    };

    let mut download_rate = 0;
    let mut upload_rate = 0;
    let mut connected_devices = 0;

    if let Some(conn_resp) = connections {
        for conn in conn_resp.connections.values() {
            if conn.connected {
                download_rate += conn.in_bytes_total; // NOTE: The API might return total bytes or rate depending on version
                upload_rate += conn.out_bytes_total;
                connected_devices += 1;
            }
        }
    }

    Ok(InstanceStatus {
        name,
        status: health.status,
        version: health.version,
        latency_ms: health.latency_ms,
        uptime: health.uptime,
        memory_alloc: health.memory_alloc,
        download_rate,
        upload_rate,
        connected_devices,
        in_sync_folders: 0,
        total_folders: 0,
    })
}

fn calculate_overall_completion(in_sync: u64, need: u64) -> f64 {
    let total = in_sync + need;
    if total > 0 {
        (in_sync as f64 / total as f64) * 100.0
    } else {
        100.0
    }
}

fn format_dashboard_text(dashboard: &GlobalDashboard) -> String {
    let mut text = String::from("## 🌐 Global SyncThing Dashboard\n\n");

    text.push_str(&format!(
        "**Network Status**: {}/{} instances online\n",
        dashboard.summary.online_instances, dashboard.summary.total_instances
    ));
    text.push_str(&format!(
        "**Aggregated Bandwidth**: ↓ {}/s | ↑ {}/s\n\n",
        format_size(dashboard.summary.total_download_rate),
        format_size(dashboard.summary.total_upload_rate)
    ));

    text.push_str("### 🖥️ Instances\n");
    for instance in &dashboard.instances {
        let status_emoji = if instance.status == "Online" {
            "🟢"
        } else {
            "🔴"
        };
        text.push_str(&format!(
            "- {} **{}** ({}ms) | ↓ {}/s | ↑ {}/s | Devices: {}\n",
            status_emoji,
            instance.name,
            instance.latency_ms,
            format_size(instance.download_rate),
            format_size(instance.upload_rate),
            instance.connected_devices
        ));
    }

    if !dashboard.alerts.is_empty() {
        text.push_str("\n### ⚠️ Active Alerts\n");
        for alert in &dashboard.alerts {
            text.push_str(&format!("- **{}**: {}\n", alert.instance, alert.message));
        }
    }

    text
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
