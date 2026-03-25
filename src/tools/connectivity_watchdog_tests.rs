use crate::tools::connectivity_watchdog::{ConnectivityMonitor, ConnectivityThresholds};
use std::time::{Duration, Instant};

#[test]
fn test_monitor_detects_offline_device() {
    let thresholds = ConnectivityThresholds {
        max_offline_duration: Duration::from_secs(300),
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
