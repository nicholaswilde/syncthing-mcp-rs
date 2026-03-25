use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Thresholds for determining if a device is offline too long and retry strategies.
#[derive(Debug, Clone)]
pub struct ConnectivityThresholds {
    /// Maximum duration a device can be offline before triggering an alert.
    pub max_offline_duration: Duration,
    /// Initial delay before the first reconnection attempt.
    pub initial_retry_delay: Duration,
    /// Maximum delay between reconnection attempts.
    pub max_retry_delay: Duration,
}

impl Default for ConnectivityThresholds {
    fn default() -> Self {
        Self {
            max_offline_duration: Duration::from_secs(300),
            initial_retry_delay: Duration::from_secs(60),
            max_retry_delay: Duration::from_secs(3600),
        }
    }
}

/// A snapshot of a device's connection status.
#[derive(Debug, Clone)]
pub struct ConnectionStatusSnapshot {
    /// Whether the device was connected.
    pub connected: bool,
    /// The time the snapshot was taken.
    pub timestamp: Instant,
    /// Number of retry attempts made since last connected.
    pub retry_count: u32,
    /// The time of the last retry attempt.
    pub last_retry: Option<Instant>,
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
        if let Some(snapshot) = self.history.get_mut(device_id) {
            if connected && !snapshot.connected {
                // Reset retry count on reconnection
                snapshot.retry_count = 0;
                snapshot.last_retry = None;
            }
            snapshot.connected = connected;
            snapshot.timestamp = now;
        } else {
            self.history.insert(
                device_id.to_string(),
                ConnectionStatusSnapshot {
                    connected,
                    timestamp: now,
                    retry_count: 0,
                    last_retry: None,
                },
            );
        }
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

    /// Checks if a retry attempt should be made for a device.
    pub fn should_retry(&self, device_id: &str, now: Instant) -> bool {
        if let Some(snapshot) = self.history.get(device_id) {
            if !snapshot.connected {
                let offline_duration = now.duration_since(snapshot.timestamp);
                if offline_duration >= self.thresholds.max_offline_duration {
                    if let Some(last_retry) = snapshot.last_retry {
                        let backoff = self.get_backoff(snapshot.retry_count);
                        return now.duration_since(last_retry) >= backoff;
                    } else {
                        // No retry attempted yet
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Records a retry attempt for a device.
    pub fn record_retry(&mut self, device_id: &str, now: Instant) {
        if let Some(snapshot) = self.history.get_mut(device_id) {
            snapshot.retry_count += 1;
            snapshot.last_retry = Some(now);
        }
    }

    fn get_backoff(&self, retry_count: u32) -> Duration {
        if retry_count == 0 {
            return Duration::from_secs(0);
        }
        let exponent = (retry_count - 1).min(10); // Cap exponent to avoid overflow
        let delay = self.thresholds.initial_retry_delay.as_secs() * (2u64.pow(exponent));
        Duration::from_secs(delay.min(self.thresholds.max_retry_delay.as_secs()))
    }

    /// Gets a list of alerts for all devices currently offline too long.
    pub fn get_alerts(&self) -> Vec<ConnectivityAlert> {
        self.alerts.clone()
    }
}
