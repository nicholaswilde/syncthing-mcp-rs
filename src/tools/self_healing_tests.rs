use crate::api::models::FolderStatus;
use crate::tools::self_healing::{
    FolderStatusSnapshot, StuckFolderMonitor, StuckFolderThresholds, check_stuck_folder,
};
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
    assert!(
        result.is_stuck,
        "Monitor should detect stuck folder after stalled period"
    );

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
    assert_eq!(
        alerts.len(),
        0,
        "Alert should be cleared after progress is made"
    );
}

#[test]
fn test_stuck_folder_detection_progress_stalled() {
    let thresholds = StuckFolderThresholds {
        max_sync_duration: Duration::from_secs(3600),
        max_stalled_duration: Duration::from_secs(300),
        max_scanning_duration: Duration::from_secs(600),
        min_rescan_interval: Duration::from_secs(300),
    };

    let base = Instant::now();

    // Initial status: syncing at 50%
    let initial_status = FolderStatus {
        state: "syncing".to_string(),
        need_bytes: 1000,
        in_sync_bytes: 500,
        ..Default::default()
    };

    let snapshot = FolderStatusSnapshot {
        status: initial_status.clone(),
        timestamp: base,
        last_rescan: None,
    };

    // Current status: same as in the snapshot
    let current_status = initial_status;

    // Check after 301 seconds
    let check_time = base + Duration::from_secs(301);
    let result = check_stuck_folder(&current_status, Some(&snapshot), &thresholds, check_time);
    assert!(
        result.is_stuck,
        "Folder should be detected as stuck due to stalled progress"
    );
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

    let base = Instant::now();

    // Initial status: scanning
    let initial_status = FolderStatus {
        state: "scanning".to_string(),
        ..Default::default()
    };

    let snapshot = FolderStatusSnapshot {
        status: initial_status.clone(),
        timestamp: base,
        last_rescan: None,
    };

    // Current status: still scanning
    let current_status = initial_status;

    // Check after 601 seconds
    let check_time = base + Duration::from_secs(601);
    let result = check_stuck_folder(&current_status, Some(&snapshot), &thresholds, check_time);
    assert!(
        result.is_stuck,
        "Folder should be detected as stuck due to long scanning"
    );
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

    let base = Instant::now();

    // Initial status: syncing at 50%
    let initial_status = FolderStatus {
        state: "syncing".to_string(),
        need_bytes: 1000,
        in_sync_bytes: 500,
        ..Default::default()
    };

    let snapshot = FolderStatusSnapshot {
        status: initial_status,
        timestamp: base,
        last_rescan: None,
    };

    // Current status: syncing at 60%
    let current_status = FolderStatus {
        state: "syncing".to_string(),
        need_bytes: 1000,
        in_sync_bytes: 600,
        ..Default::default()
    };

    // Check after 301 seconds
    let check_time = base + Duration::from_secs(301);
    let result = check_stuck_folder(&current_status, Some(&snapshot), &thresholds, check_time);
    assert!(
        !result.is_stuck,
        "Folder should NOT be detected as stuck because progress was made"
    );
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

#[test]
fn test_check_stuck_folder_no_previous() {
    let thresholds = StuckFolderThresholds::default();
    let now = Instant::now();
    let current = FolderStatus {
        state: "syncing".to_string(),
        ..Default::default()
    };

    let result = check_stuck_folder(&current, None, &thresholds, now);
    assert!(!result.is_stuck);
}

#[test]
fn test_monitor_should_rescan_no_alert() {
    let thresholds = StuckFolderThresholds::default();
    let mut monitor = StuckFolderMonitor::new(thresholds);
    let now = Instant::now();

    monitor.update("folder1", FolderStatus::default(), now);
    assert!(!monitor.should_rescan("folder1", now));
}

#[test]
fn test_monitor_should_rescan_no_history() {
    let thresholds = StuckFolderThresholds::default();
    let monitor = StuckFolderMonitor::new(thresholds);
    let now = Instant::now();

    assert!(!monitor.should_rescan("folder1", now));
}

#[tokio::test]
async fn test_monitor_self_healing_full() {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::self_healing::monitor_self_healing;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;

    // Mock list_folders
    Mock::given(method("GET"))
        .and(path("/rest/config/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{"id": "folder1", "path": "/tmp", "label": "Folder 1", "type": "sendreceive", "devices": []}])))
        .mount(&server)
        .await;

    // Mock get_folder_status
    Mock::given(method("GET"))
        .and(path("/rest/db/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "state": "syncing",
            "inSyncBytes": 500,
            "needBytes": 1000
        })))
        .mount(&server)
        .await;

    // Mock rescan
    Mock::given(method("POST"))
        .and(path("/rest/db/scan"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    // Mock get_connections
    Mock::given(method("GET"))
        .and(path("/rest/system/connections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "connections": {
                "device1": {
                    "connected": false,
                    "paused": false,
                    "type": "tcp-client"
                }
            }
        })))
        .mount(&server)
        .await;

    // Mock patch_device (pause/resume)
    Mock::given(method("PATCH"))
        .and(path("/rest/config/devices/device1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = SyncThingClient::new(InstanceConfig {
        url: server.uri(),
        api_key: Some("test".to_string()),
        ..Default::default()
    });
    let config = AppConfig::default();

    // 1. Initial call to establish history
    let _ = monitor_self_healing(client.clone(), config.clone(), json!({"dry_run": false}))
        .await
        .unwrap();

    // Note: We can't easily wait for 300s in a unit test without mocking Instant,
    // which our implementation doesn't support easily as it uses Instant::now() directly.
    // However, the check() function is called, which increases coverage of the monitor logic.

    let result = monitor_self_healing(client, config, json!({"dry_run": false}))
        .await
        .unwrap();
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Self-Healing Monitor Report:"));
}

#[tokio::test]
async fn test_monitor_self_healing_dry_run() {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::self_healing::monitor_self_healing;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;

    // Mock list_folders
    Mock::given(method("GET"))
        .and(path("/rest/config/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{"id": "folder1", "path": "/tmp", "label": "Folder 1", "type": "sendreceive", "devices": []}])))
        .mount(&server)
        .await;

    // Mock get_folder_status
    Mock::given(method("GET"))
        .and(path("/rest/db/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "state": "syncing",
            "inSyncBytes": 500,
            "needBytes": 1000
        })))
        .mount(&server)
        .await;

    // Mock get_connections
    Mock::given(method("GET"))
        .and(path("/rest/system/connections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "connections": {
                "device1": {
                    "connected": false,
                    "paused": false,
                    "type": "tcp-client"
                }
            }
        })))
        .mount(&server)
        .await;

    let client = SyncThingClient::new(InstanceConfig {
        url: server.uri(),
        api_key: Some("test".to_string()),
        ..Default::default()
    });
    let config = AppConfig::default();

    // First call to establish history
    let _ = monitor_self_healing(client.clone(), config.clone(), json!({"dry_run": true}))
        .await
        .unwrap();

    // Fast-forward time is not easy with real Instant, but we can call it again
    // and since it's dry_run it shouldn't trigger anything yet because duration is 0
    let result = monitor_self_healing(client, config, json!({"dry_run": true}))
        .await
        .unwrap();
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Self-Healing Monitor Report:"));
    assert!(text.contains("No actions needed at this time."));
}
