use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;

/// Bandwidth limits to apply.
#[derive(Debug, Clone, Default)]
pub struct BandwidthLimits {
    /// Maximum receive rate in Kbps.
    pub max_recv_kbps: Option<i64>,
    /// Maximum send rate in Kbps.
    pub max_send_kbps: Option<i64>,
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
