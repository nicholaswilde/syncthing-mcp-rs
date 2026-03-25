use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Thresholds for determining if a device is offline too long.
#[derive(Debug, Clone)]
pub struct ConnectivityThresholds {
    /// Maximum duration a device can be offline before triggering an alert.
    pub max_offline_duration: Duration,
}

/// A snapshot of a device's connection status.
#[derive(Debug, Clone)]
pub struct ConnectionStatusSnapshot {
    /// Whether the device was connected.
    pub connected: bool,
    /// The time the snapshot was taken.
    pub timestamp: Instant,
}

/// Result of a connectivity check.
pub struct ConnectivityCheckResult {
    /// Whether the device is considered offline too long.
    pub is_offline_too_long: bool,
    /// The reason for the status.
    pub reason: Option<String>,
}

/// Alert for a device that is offline too long.
#[derive(Debug, Clone)]
pub struct ConnectivityAlert {
    /// The device ID.
    pub device_id: String,
    /// The reason for the alert.
    pub reason: String,
    /// The time the alert was generated.
    pub timestamp: Instant,
}

/// Monitor that tracks device connectivity over time.
pub struct ConnectivityMonitor {
    /// Thresholds for connectivity alerts.
    pub thresholds: ConnectivityThresholds,
    /// Map of device IDs to their last known connection status and timestamp.
    pub history: HashMap<String, ConnectionStatusSnapshot>,
    /// List of current alerts.
    pub alerts: Vec<ConnectivityAlert>,
}

impl ConnectivityMonitor {
    /// Creates a new ConnectivityMonitor with the given thresholds.
    pub fn new(thresholds: ConnectivityThresholds) -> Self {
        Self {
            thresholds,
            history: HashMap::new(),
            alerts: Vec::new(),
        }
    }

    /// Updates the history with the latest connection status.
    pub fn update(&mut self, device_id: &str, connected: bool, now: Instant) {
        self.history.insert(
            device_id.to_string(),
            ConnectionStatusSnapshot { connected, timestamp: now },
        );
    }

    /// Checks if a device is offline too long.
    pub fn check(
        &mut self,
        device_id: &str,
        connected: bool,
        now: Instant,
    ) -> ConnectivityCheckResult {
        let is_offline_too_long;
        let mut reason = None;

        match self.history.get(device_id) {
            Some(prev) if !connected && !prev.connected => {
                // Still offline, check duration
                let duration = now.duration_since(prev.timestamp);
                if duration >= self.thresholds.max_offline_duration {
                    is_offline_too_long = true;
                    reason = Some(format!("Device offline for {}s", duration.as_secs()));
                } else {
                    is_offline_too_long = false;
                }
            }
            _ => {
                // Any other case: Connected, or first time offline, or transitioned
                self.update(device_id, connected, now);
                is_offline_too_long = false;
            }
        }

        if is_offline_too_long {
            if let Some(r) = &reason {
                if let Some(alert) = self.alerts.iter_mut().find(|a| a.device_id == device_id) {
                    alert.reason = r.clone();
                    alert.timestamp = now;
                } else {
                    self.alerts.push(ConnectivityAlert {
                        device_id: device_id.to_string(),
                        reason: r.clone(),
                        timestamp: now,
                    });
                }
            }
        } else {
            self.alerts.retain(|a| a.device_id != device_id);
        }

        ConnectivityCheckResult {
            is_offline_too_long,
            reason,
        }
    }

    /// Gets a list of alerts for all devices currently offline too long.
    pub fn get_alerts(&self) -> Vec<ConnectivityAlert> {
        self.alerts.clone()
    }
}
