use crate::api::SyncThingClient;
use crate::config::{AppConfig, BandwidthConfig, BandwidthLimits};
use crate::error::Result;

use serde_json::Value;

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

    /// Gets the scheduled profile for a given time.
    pub fn get_scheduled_profile_at(&self, now: chrono::NaiveDateTime) -> Option<String> {
        let day = now.format("%A").to_string();
        let time = now.format("%H:%M").to_string();

        for schedule in &self.config.schedules {
            if schedule.days.contains(&day) && time >= schedule.start_time && time <= schedule.end_time {
                return Some(schedule.profile_name.clone());
            }
        }
        None
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

/// MCP tool to set bandwidth limits.
pub async fn set_bandwidth_limits(
    _client: SyncThingClient,
    app_config: AppConfig,
    params: Value,
) -> Result<Value> {
    let limits: BandwidthLimits = serde_json::from_value(params.clone())?;
    let target_instance = params.get("instance").and_then(|v| v.as_str());

    let controller = BandwidthController::new();
    controller
        .update_bandwidth_limits(&app_config, target_instance, limits)
        .await?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": "Bandwidth limits updated successfully"
        }]
    }))
}

/// MCP tool to set the active performance profile.
pub async fn set_performance_profile(
    _client: SyncThingClient,
    app_config: AppConfig,
    params: Value,
) -> Result<Value> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::error::Error::Internal("name is required".to_string()))?;

    let mut manager = ProfileManager::new(app_config.bandwidth.clone());
    let limits = manager
        .apply_profile(name)
        .ok_or_else(|| crate::error::Error::Internal(format!("Profile not found: {}", name)))?;

    let controller = BandwidthController::new();
    controller
        .update_bandwidth_limits(&app_config, None, limits)
        .await?;

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": format!("Performance profile '{}' applied successfully", name)
        }]
    }))
}

/// MCP tool to get the current bandwidth status.
pub async fn get_bandwidth_status(
    _client: SyncThingClient,
    app_config: AppConfig,
    _params: Value,
) -> Result<Value> {
    let mut instance_stats = Vec::new();

    for (i, instance_config) in app_config.instances.iter().enumerate() {
        let name = instance_config
            .name
            .clone()
            .unwrap_or_else(|| format!("Instance {}", i));
        
        let client = SyncThingClient::new(instance_config.clone());
        let config = client.get_config().await?;
        
        let max_recv = config.options.get("maxRecvKbps")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let max_send = config.options.get("maxSendKbps")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
            
        instance_stats.push(format!(
            "- Instance {}: Recv {} Kbps, Send {} Kbps",
            name,
            if max_recv == 0 { "Unlimited".to_string() } else { max_recv.to_string() },
            if max_send == 0 { "Unlimited".to_string() } else { max_send.to_string() }
        ));
    }

    let mut text = String::from("Bandwidth Status:\n\n");
    text.push_str(&instance_stats.join("\n"));
    text.push_str("\n\nActive Profile: none"); // TODO: Implement persistent active profile tracking

    Ok(serde_json::json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}
