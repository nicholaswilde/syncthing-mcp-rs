use crate::api::models::FolderStatus;
use crate::tools::self_healing::{check_stuck_folder, StuckFolderThresholds, FolderStatusSnapshot};
use std::time::{Duration, Instant};

#[test]
fn test_stuck_folder_detection_progress_stalled() {
    let thresholds = StuckFolderThresholds {
        max_sync_duration: Duration::from_secs(3600),
        max_stalled_duration: Duration::from_secs(300),
        max_scanning_duration: Duration::from_secs(600),
    };

    let now = Instant::now();
    
    // Initial status: syncing at 50%
    let initial_status = FolderStatus {
        state: "syncing".to_string(),
        need_bytes: 1000,
        in_sync_bytes: 500,
        ..Default::default()
    };
    
    let snapshot = FolderStatusSnapshot {
        status: initial_status.clone(),
        timestamp: now - Duration::from_secs(301),
    };

    // Current status: same as 301 seconds ago
    let current_status = initial_status;
    
    let result = check_stuck_folder(&current_status, Some(&snapshot), &thresholds, now);
    assert!(result.is_stuck, "Folder should be detected as stuck due to stalled progress");
    assert_eq!(result.reason, Some("Progress stalled for 301s".to_string()));
}

#[test]
fn test_stuck_folder_detection_scanning_too_long() {
    let thresholds = StuckFolderThresholds {
        max_sync_duration: Duration::from_secs(3600),
        max_stalled_duration: Duration::from_secs(300),
        max_scanning_duration: Duration::from_secs(600),
    };

    let now = Instant::now();
    
    // Initial status: scanning
    let initial_status = FolderStatus {
        state: "scanning".to_string(),
        ..Default::default()
    };
    
    let snapshot = FolderStatusSnapshot {
        status: initial_status.clone(),
        timestamp: now - Duration::from_secs(601),
    };

    // Current status: still scanning
    let current_status = initial_status;
    
    let result = check_stuck_folder(&current_status, Some(&snapshot), &thresholds, now);
    assert!(result.is_stuck, "Folder should be detected as stuck due to long scanning");
    assert_eq!(result.reason, Some("Scanning for 601s".to_string()));
}

#[test]
fn test_not_stuck_if_progress_made() {
    let thresholds = StuckFolderThresholds {
        max_sync_duration: Duration::from_secs(3600),
        max_stalled_duration: Duration::from_secs(300),
        max_scanning_duration: Duration::from_secs(600),
    };

    let now = Instant::now();
    
    // Initial status: syncing at 50%
    let initial_status = FolderStatus {
        state: "syncing".to_string(),
        need_bytes: 1000,
        in_sync_bytes: 500,
        ..Default::default()
    };
    
    let snapshot = FolderStatusSnapshot {
        status: initial_status,
        timestamp: now - Duration::from_secs(301),
    };

    // Current status: syncing at 60%
    let current_status = FolderStatus {
        state: "syncing".to_string(),
        need_bytes: 1000,
        in_sync_bytes: 600,
        ..Default::default()
    };
    
    let result = check_stuck_folder(&current_status, Some(&snapshot), &thresholds, now);
    assert!(!result.is_stuck, "Folder should NOT be detected as stuck because progress was made");
}
