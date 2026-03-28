use crate::api::SyncThingClient;
use crate::api::models::FolderStatus;
use crate::config::AppConfig;
use crate::error::Result;
use crate::tools::connectivity_watchdog::{ConnectivityMonitor, ConnectivityThresholds};
use lazy_static::lazy_static;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

lazy_static! {
    static ref STUCK_FOLDER_MONITOR: Mutex<StuckFolderMonitor> =
        Mutex::new(StuckFolderMonitor::new(StuckFolderThresholds::default()));
    static ref CONNECTIVITY_MONITOR: Mutex<ConnectivityMonitor> =
        Mutex::new(ConnectivityMonitor::new(ConnectivityThresholds::default()));
}

/// Thresholds for determining if a folder is stuck.
#[derive(Debug, Clone)]
pub struct StuckFolderThresholds {
    /// Maximum duration a folder can be in a "syncing" state without progress.
    pub max_stalled_duration: Duration,
    /// Maximum duration a folder can be in a "scanning" state.
    pub max_scanning_duration: Duration,
    /// Maximum total duration for a sync operation.
    pub max_sync_duration: Duration,
    /// Minimum interval between automatic rescans.
    pub min_rescan_interval: Duration,
}

impl Default for StuckFolderThresholds {
    fn default() -> Self {
        Self {
            max_stalled_duration: Duration::from_secs(300),
            max_scanning_duration: Duration::from_secs(600),
            max_sync_duration: Duration::from_secs(3600),
            min_rescan_interval: Duration::from_secs(300),
        }
    }
}

/// A snapshot of a folder's status at a specific time.
#[derive(Debug, Clone)]
pub struct FolderStatusSnapshot {
    /// The folder status at the time of the snapshot.
    pub status: FolderStatus,
    /// The time the snapshot was taken.
    pub timestamp: Instant,
    /// The time of the last automatic rescan.
    pub last_rescan: Option<Instant>,
}

/// Result of a stuck folder check.
pub struct StuckCheckResult {
    /// Whether the folder is considered stuck.
    pub is_stuck: bool,
    /// The reason why the folder is considered stuck.
    pub reason: Option<String>,
}

/// Alert for a folder that is stuck.
#[derive(Debug, Clone)]
pub struct FolderAlert {
    /// The folder ID.
    pub folder_id: String,
    /// The reason for the alert.
    pub reason: String,
    /// The time the alert was generated.
    pub timestamp: Instant,
}

/// Monitor that tracks folder status over time to detect stuck operations.
pub struct StuckFolderMonitor {
    /// Thresholds for determining if a folder is stuck.
    pub thresholds: StuckFolderThresholds,
    /// Map of folder IDs to their last known status and timestamp.
    pub history: HashMap<String, FolderStatusSnapshot>,
    /// List of current alerts.
    pub alerts: Vec<FolderAlert>,
}

impl StuckFolderMonitor {
    /// Creates a new StuckFolderMonitor with the given thresholds.
    pub fn new(thresholds: StuckFolderThresholds) -> Self {
        Self {
            thresholds,
            history: HashMap::new(),
            alerts: Vec::new(),
        }
    }

    /// Updates the history with the latest folder status.
    pub fn update(&mut self, folder_id: &str, status: FolderStatus, now: Instant) {
        if let Some(snapshot) = self.history.get_mut(folder_id) {
            snapshot.status = status;
            snapshot.timestamp = now;
        } else {
            self.history.insert(
                folder_id.to_string(),
                FolderStatusSnapshot {
                    status,
                    timestamp: now,
                    last_rescan: None,
                },
            );
        }
    }

    /// Checks if a folder is stuck based on current status and its history in the monitor.
    #[allow(clippy::collapsible_if)]
    pub fn check(
        &mut self,
        folder_id: &str,
        current: FolderStatus,
        now: Instant,
    ) -> StuckCheckResult {
        let previous = self.history.get(folder_id);
        let result = check_stuck_folder(&current, previous, &self.thresholds, now);

        if result.is_stuck {
            if let Some(reason) = &result.reason {
                // Update alert or add new one
                if let Some(alert) = self.alerts.iter_mut().find(|a| a.folder_id == folder_id) {
                    alert.reason = reason.clone();
                    alert.timestamp = now;
                } else {
                    self.alerts.push(FolderAlert {
                        folder_id: folder_id.to_string(),
                        reason: reason.clone(),
                        timestamp: now,
                    });
                }
            }
        } else {
            // Remove alert if it was cleared
            self.alerts.retain(|a| a.folder_id != folder_id);
        }

        // Always update history with latest status
        self.update(folder_id, current, now);

        result
    }

    /// Checks if an automatic rescan should be triggered for a folder.
    #[allow(clippy::collapsible_if)]
    pub fn should_rescan(&self, folder_id: &str, now: Instant) -> bool {
        if let Some(snapshot) = self.history.get(folder_id) {
            // Check if folder is currently in an alert state
            if self.alerts.iter().any(|a| a.folder_id == folder_id) {
                if let Some(last_rescan) = snapshot.last_rescan {
                    return now.duration_since(last_rescan) >= self.thresholds.min_rescan_interval;
                } else {
                    return true;
                }
            }
        }
        false
    }

    /// Records that an automatic rescan was triggered for a folder.
    pub fn record_rescan(&mut self, folder_id: &str, now: Instant) {
        if let Some(snapshot) = self.history.get_mut(folder_id) {
            snapshot.last_rescan = Some(now);
        }
    }

    /// Gets a list of alerts for all folders currently deemed stuck.
    pub fn get_alerts(&self, _now: Instant) -> Vec<FolderAlert> {
        self.alerts.clone()
    }
}

/// Checks if a folder is stuck based on current status and history.
#[allow(clippy::collapsible_if)]
pub fn check_stuck_folder(
    current: &FolderStatus,
    previous: Option<&FolderStatusSnapshot>,
    thresholds: &StuckFolderThresholds,
    now: Instant,
) -> StuckCheckResult {
    if let Some(prev) = previous {
        let duration = now.duration_since(prev.timestamp);

        // Check if scanning for too long
        if current.state == "scanning"
            && prev.status.state == "scanning"
            && duration >= thresholds.max_scanning_duration
        {
            return StuckCheckResult {
                is_stuck: true,
                reason: Some(format!("Scanning for {}s", duration.as_secs())),
            };
        }

        // Check if progress stalled during syncing
        if current.state == "syncing"
            && prev.status.state == "syncing"
            && current.in_sync_bytes == prev.status.in_sync_bytes
            && duration >= thresholds.max_stalled_duration
        {
            return StuckCheckResult {
                is_stuck: true,
                reason: Some(format!("Progress stalled for {}s", duration.as_secs())),
            };
        }
    }

    StuckCheckResult {
        is_stuck: false,
        reason: None,
    }
}

/// Monitor tool that checks for stuck folders and disconnected devices, and triggers self-healing actions.
pub async fn monitor_self_healing(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let now = Instant::now();
    let dry_run = args["dry_run"].as_bool().unwrap_or(false);
    let mut actions_taken = Vec::new();

    // 1. Fetch data for folders (without holding lock)
    let folders = client.list_folders().await?;
    let mut folder_statuses = HashMap::new();
    for folder_cfg in folders {
        if let Ok(status) = client.get_folder_status(&folder_cfg.id).await {
            folder_statuses.insert(folder_cfg.id, status);
        }
    }

    // 2. Perform folder checks (holding lock briefly)
    let mut rescan_targets = Vec::new();
    {
        let mut monitor = STUCK_FOLDER_MONITOR.lock().unwrap();
        for (id, status) in folder_statuses {
            monitor.check(&id, status, now);
            if monitor.should_rescan(&id, now) {
                rescan_targets.push(id);
            }
        }
    }

    // 3. Trigger folder actions (without holding lock)
    for id in rescan_targets {
        if !dry_run {
            if let Err(e) = client.rescan(Some(&id)).await {
                tracing::error!("Failed to rescan folder {}: {}", id, e);
            } else {
                STUCK_FOLDER_MONITOR.lock().unwrap().record_rescan(&id, now);
                actions_taken.push(format!("Triggered rescan for folder {}", id));
            }
        } else {
            actions_taken.push(format!("[Dry Run] Would trigger rescan for folder {}", id));
        }
    }

    // 4. Fetch data for connections (without holding lock)
    let connections = client.get_connections().await?;

    // 5. Perform device checks (holding lock briefly)
    let mut reconnection_targets = Vec::new();
    {
        let mut monitor = CONNECTIVITY_MONITOR.lock().unwrap();
        for (device_id, conn_status) in connections.connections {
            monitor.check(&device_id, conn_status.connected, now);
            if monitor.should_retry(&device_id, now) {
                reconnection_targets.push(device_id);
            }
        }
    }

    // 6. Trigger device actions (without holding lock)
    for device_id in reconnection_targets {
        if !dry_run {
            // Force reconnection by pausing and resuming
            if let Err(e) = client
                .patch_device(&device_id, json!({"paused": true}))
                .await
            {
                tracing::error!("Failed to pause device {}: {}", device_id, e);
            } else if let Err(e) = client
                .patch_device(&device_id, json!({"paused": false}))
                .await
            {
                tracing::error!("Failed to resume device {}: {}", device_id, e);
            } else {
                CONNECTIVITY_MONITOR
                    .lock()
                    .unwrap()
                    .record_retry(&device_id, now);
                actions_taken.push(format!("Triggered reconnection for device {}", device_id));
            }
        } else {
            actions_taken.push(format!(
                "[Dry Run] Would trigger reconnection for device {}",
                device_id
            ));
        }
    }

    let mut text = String::from("Self-Healing Monitor Report:\n\n");
    if actions_taken.is_empty() {
        text.push_str("No actions needed at this time.\n");
    } else {
        text.push_str("Actions Taken:\n");
        for action in actions_taken {
            text.push_str(&format!("- {}\n", action));
        }
    }

    // Add current alerts
    let folder_alerts = STUCK_FOLDER_MONITOR.lock().unwrap().get_alerts(now);
    let connectivity_alerts = CONNECTIVITY_MONITOR.lock().unwrap().get_alerts();

    if !folder_alerts.is_empty() || !connectivity_alerts.is_empty() {
        text.push_str("\nCurrent Alerts:\n");
        for alert in folder_alerts {
            text.push_str(&format!(
                "- [Folder {}] {}\n",
                alert.folder_id, alert.reason
            ));
        }
        for alert in connectivity_alerts {
            text.push_str(&format!(
                "- [Device {}] {}\n",
                alert.device_id, alert.reason
            ));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text.trim_end()
        }]
    }))
}
