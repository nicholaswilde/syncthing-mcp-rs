use crate::api::models::FolderStatus;
use crate::tools::self_healing::{check_stuck_folder, StuckFolderThresholds, FolderStatusSnapshot, StuckFolderMonitor};
use std::time::{Duration, Instant};

#[test]
fn test_monitor_detects_stuck_folder() {
    let thresholds = StuckFolderThresholds {
        max_sync_duration: Duration::from_secs(3600),
        max_stalled_duration: Duration::from_secs(300),
        max_scanning_duration: Duration::from_secs(600),
        min_rescan_interval: Duration::from_secs(300),
    };

    let mut monitor = StuckFolderMonitor::new(thresholds);
    let now = Instant::now();
    
    // Initial status: syncing at 50%
    let status_t0 = FolderStatus {
        state: "syncing".to_string(),
        need_bytes: 1000,
        in_sync_bytes: 500,
        ..Default::default()
    };
    
    // Update monitor at t0
    monitor.update("folder1", status_t0.clone(), now);
    
    // Check after 301 seconds, same status
    let result = monitor.check("folder1", status_t0.clone(), now + Duration::from_secs(301));
    assert!(result.is_stuck, "Monitor should detect stuck folder after stalled period");

    // Task 3: Verify alerts
    let alerts = monitor.get_alerts(now + Duration::from_secs(301));
    assert_eq!(alerts.len(), 1, "Should have one alert for stuck folder");
    assert_eq!(alerts[0].folder_id, "folder1");
    assert_eq!(alerts[0].reason, "Progress stalled for 301s");

    // Clear alert by making progress
    let status_t2 = FolderStatus {
        state: "syncing".to_string(),
        need_bytes: 1000,
        in_sync_bytes: 600,
        ..Default::default()
    };
    monitor.check("folder1", status_t2, now + Duration::from_secs(302));
    let alerts = monitor.get_alerts(now + Duration::from_secs(302));
    assert_eq!(alerts.len(), 0, "Alert should be cleared after progress is made");
}


#[test]
fn test_stuck_folder_detection_progress_stalled() {
    let thresholds = StuckFolderThresholds {
        max_sync_duration: Duration::from_secs(3600),
        max_stalled_duration: Duration::from_secs(300),
        max_scanning_duration: Duration::from_secs(600),
        min_rescan_interval: Duration::from_secs(300),
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
        last_rescan: None,
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
        min_rescan_interval: Duration::from_secs(300),
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
        last_rescan: None,
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
        min_rescan_interval: Duration::from_secs(300),
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
        last_rescan: None,
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

#[test]
fn test_should_suggest_rescan_for_stuck_folder() {
    let thresholds = StuckFolderThresholds {
        max_sync_duration: Duration::from_secs(3600),
        max_stalled_duration: Duration::from_secs(300),
        max_scanning_duration: Duration::from_secs(600),
        min_rescan_interval: Duration::from_secs(300),
    };

    let mut monitor = StuckFolderMonitor::new(thresholds);
    let now = Instant::now();
    
    // Initial status: syncing at 50%
    let status_t0 = FolderStatus {
        state: "syncing".to_string(),
        in_sync_bytes: 500,
        ..Default::default()
    };
    monitor.update("folder1", status_t0.clone(), now);
    
    // Check after 301 seconds, same status
    monitor.check("folder1", status_t0.clone(), now + Duration::from_secs(301));
    
    // Should suggest rescan
    assert!(monitor.should_rescan("folder1", now + Duration::from_secs(301)));
    
    // Record rescan at t=301
    monitor.record_rescan("folder1", now + Duration::from_secs(301));
    
    // Next rescan should NOT be immediate
    assert!(!monitor.should_rescan("folder1", now + Duration::from_secs(302)));
}
