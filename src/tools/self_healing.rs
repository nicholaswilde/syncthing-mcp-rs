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
}

/// A snapshot of a folder's status at a specific time.
#[derive(Debug, Clone)]
pub struct FolderStatusSnapshot {
    /// The folder status at the time of the snapshot.
    pub status: FolderStatus,
    /// The time the snapshot was taken.
    pub timestamp: Instant,
}

/// Result of a stuck folder check.
pub struct StuckCheckResult {
    /// Whether the folder is considered stuck.
    pub is_stuck: bool,
    /// The reason why the folder is considered stuck.
    pub reason: Option<String>,
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
