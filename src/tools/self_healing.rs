use crate::api::models::FolderStatus;
use std::time::{Duration, Instant};

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

use std::collections::HashMap;

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
pub fn check_stuck_folder(
    current: &FolderStatus,
    previous: Option<&FolderStatusSnapshot>,
    thresholds: &StuckFolderThresholds,
    now: Instant,
) -> StuckCheckResult {
    if let Some(prev) = previous {
        let duration = now.duration_since(prev.timestamp);

        // Check if scanning for too long
        if current.state == "scanning" && prev.status.state == "scanning" {
            if duration >= thresholds.max_scanning_duration {
                return StuckCheckResult {
                    is_stuck: true,
                    reason: Some(format!("Scanning for {}s", duration.as_secs())),
                };
            }
        }

        // Check if progress stalled during syncing
        if current.state == "syncing" && prev.status.state == "syncing" {
            if current.in_sync_bytes == prev.status.in_sync_bytes {
                if duration >= thresholds.max_stalled_duration {
                    return StuckCheckResult {
                        is_stuck: true,
                        reason: Some(format!("Progress stalled for {}s", duration.as_secs())),
                    };
                }
            }
        }
    }

    StuckCheckResult {
        is_stuck: false,
        reason: None,
    }
}
