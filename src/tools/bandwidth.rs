use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;

use serde::{Deserialize, Serialize};

/// Bandwidth limits to apply.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BandwidthLimits {
    /// Maximum receive rate in Kbps.
    pub max_recv_kbps: Option<i64>,
    /// Maximum send rate in Kbps.
    pub max_send_kbps: Option<i64>,
}

/// A performance profile that defines bandwidth limits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceProfile {
    /// Name of the profile (e.g., "working_hours").
    pub name: String,
    /// Bandwidth limits for this profile.
    pub limits: BandwidthLimits,
}

/// A schedule for when to apply a performance profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileSchedule {
    /// Profile to apply.
    pub profile_name: String,
    /// Days of the week this schedule applies to.
    pub days: Vec<String>,
    /// Start time in 24h format (e.g., "09:00").
    pub start_time: String,
    /// End time in 24h format (e.g., "17:00").
    pub end_time: String,
}

/// Configuration for bandwidth orchestration.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BandwidthConfig {
    /// Available performance profiles.
    pub profiles: Vec<PerformanceProfile>,
    /// Schedules for applying profiles.
    pub schedules: Vec<ProfileSchedule>,
    /// The name of the currently active profile, if any.
    pub active_profile: Option<String>,
}

/// Manager for performance profiles.
pub struct ProfileManager {
    /// Current bandwidth configuration.
    pub config: BandwidthConfig,
}

impl ProfileManager {
    /// Creates a new `ProfileManager`.
    pub fn new(config: BandwidthConfig) -> Self {
        Self { config }
    }

    /// Applies a performance profile by name.
    /// Returns the bandwidth limits for the profile if found.
    pub fn apply_profile(&mut self, name: &str) -> Option<BandwidthLimits> {
        let profile = self.config.profiles.iter().find(|p| p.name == name)?;
        self.config.active_profile = Some(name.to_string());
        Some(profile.limits.clone())
    }
}

/// Controller for managing bandwidth limits.
pub struct BandwidthController;

impl BandwidthController {
    /// Creates a new `BandwidthController`.
    pub fn new() -> Self {
        Self
    }

    /// Sets the bandwidth limits for a single instance.
    pub async fn set_instance_bandwidth_limits(
        &self,
        client: &SyncThingClient,
        limits: BandwidthLimits,
    ) -> Result<()> {
        let mut config = client.get_config().await?;

        if let Some(recv) = limits.max_recv_kbps {
            config.options["maxRecvKbps"] = serde_json::json!(recv);
        }
        if let Some(send) = limits.max_send_kbps {
            config.options["maxSendKbps"] = serde_json::json!(send);
        }

        client.set_config(config).await?;
        Ok(())
    }

    /// Sets the bandwidth limits for one or all instances.
    pub async fn update_bandwidth_limits(
        &self,
        app_config: &AppConfig,
        target_instance: Option<&str>,
        limits: BandwidthLimits,
    ) -> Result<()> {
        for (i, instance_config) in app_config.instances.iter().enumerate() {
            let name = instance_config
                .name
                .clone()
                .unwrap_or_else(|| format!("Instance {}", i));
            
            // If target_instance is specified, skip if it doesn't match the name or index
            if let Some(target) = target_instance {
                if target != name && target != i.to_string() {
                    continue;
                }
            }

            let client = SyncThingClient::new(instance_config.clone());
            self.set_instance_bandwidth_limits(&client, limits.clone()).await?;
        }

        Ok(())
    }
}
