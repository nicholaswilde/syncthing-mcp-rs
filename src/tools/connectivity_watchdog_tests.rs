use crate::tools::connectivity_watchdog::{ConnectivityMonitor, ConnectivityThresholds};
use std::time::{Duration, Instant};

#[test]
fn test_monitor_detects_offline_device() {
    let thresholds = ConnectivityThresholds {
        max_offline_duration: Duration::from_secs(300),
        initial_retry_delay: Duration::from_secs(60),
        max_retry_delay: Duration::from_secs(3600),
    };

    let mut monitor = ConnectivityMonitor::new(thresholds);
    let now = Instant::now();
    
    // Initial status: connected at t=0
    monitor.update("device1", true, now);
    
    // Check at t=1, detected as disconnected. 
    // This starts the offline timer at t=1.
    let result = monitor.check("device1", false, now + Duration::from_secs(1));
    assert!(!result.is_offline_too_long, "Should not be offline too long immediately after disconnect");

    // Check at t=302 (301 seconds after disconnect at t=1)
    let result = monitor.check("device1", false, now + Duration::from_secs(302));
    assert!(result.is_offline_too_long, "Monitor should detect offline device after threshold");

    // Verify alerts
    let alerts = monitor.get_alerts();
    assert_eq!(alerts.len(), 1, "Should have one alert for offline device");
    assert_eq!(alerts[0].device_id, "device1");
    assert_eq!(alerts[0].reason, "Device offline for 301s");

    // Clear alert by reconnecting
    monitor.check("device1", true, now + Duration::from_secs(303));
    let alerts = monitor.get_alerts();
    assert_eq!(alerts.len(), 0, "Alert should be cleared after device reconnects");
}

#[test]
fn test_exponential_backoff_retries() {
    let thresholds = ConnectivityThresholds {
        max_offline_duration: Duration::from_secs(300),
        initial_retry_delay: Duration::from_secs(60),
        max_retry_delay: Duration::from_secs(3600),
    };

    let mut monitor = ConnectivityMonitor::new(thresholds);
    let now = Instant::now();
    
    // Initial status: connected at t=0
    monitor.update("device1", true, now);
    
    // Disconnect at t=1
    monitor.check("device1", false, now + Duration::from_secs(1));
    
    // Check at t=302 (offline for 301s)
    let result = monitor.check("device1", false, now + Duration::from_secs(302));
    assert!(result.is_offline_too_long);
    
    // First retry should be suggested immediately when offline too long
    assert!(monitor.should_retry("device1", now + Duration::from_secs(302)));
    
    // Record retry at t=302 (retry_count = 1)
    monitor.record_retry("device1", now + Duration::from_secs(302));
    
    // Next retry should NOT be immediate (backoff 60s)
    assert!(!monitor.should_retry("device1", now + Duration::from_secs(303)));
    
    // Next retry at t=302 + 60s
    assert!(monitor.should_retry("device1", now + Duration::from_secs(302 + 61)));

    // Record second retry at t=363 (retry_count = 2)
    monitor.record_retry("device1", now + Duration::from_secs(363));

    // Next retry should be after 120s (60 * 2^1)
    assert!(!monitor.should_retry("device1", now + Duration::from_secs(363 + 119)));
    assert!(monitor.should_retry("device1", now + Duration::from_secs(363 + 121)));
}

#[test]
fn test_no_retry_if_all_devices_down() {
    let thresholds = ConnectivityThresholds {
        max_offline_duration: Duration::from_secs(300),
        initial_retry_delay: Duration::from_secs(60),
        max_retry_delay: Duration::from_secs(3600),
    };

    let mut monitor = ConnectivityMonitor::new(thresholds);
    let now = Instant::now();
    
    // Multiple devices in history
    monitor.update("device1", true, now);
    monitor.update("device2", true, now);
    
    // Both go down
    monitor.check("device1", false, now + Duration::from_secs(1));
    monitor.check("device2", false, now + Duration::from_secs(1));
    
    // Check at t=302 (offline for 301s)
    monitor.check("device1", false, now + Duration::from_secs(302));
    monitor.check("device2", false, now + Duration::from_secs(302));
    
    // is_all_offline should be true
    assert!(monitor.is_all_offline());
    
    // Should NOT retry because all devices are down (likely network issue)
    assert!(!monitor.should_retry("device1", now + Duration::from_secs(302)));
}
