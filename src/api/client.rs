use crate::api::models::*;
use crate::config::InstanceConfig;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::time::Duration;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

/// A client for interacting with the SyncThing REST API.
#[derive(Debug, Clone)]
pub struct SyncThingClient {
    /// The HTTP client used for requests.
    pub client: reqwest::Client,
    /// The configuration for the SyncThing instance.
    pub config: InstanceConfig,
}

impl SyncThingClient {
    /// Creates a new SyncThing client with the given configuration.
    pub fn new(config: InstanceConfig) -> Self {
        let timeout = Duration::from_secs(config.timeout_s.unwrap_or(30));
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.no_verify_ssl.unwrap_or(true))
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client, config }
    }

    fn add_auth(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(api_key) = &self.config.api_key {
            request = request.header("X-API-Key", api_key);
        }
        request
    }

    async fn send_with_retry(
        &self,
        request_builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let max_attempts = self.config.retry_max_attempts.unwrap_or(3) as usize;
        let initial_backoff = self.config.retry_initial_backoff_ms.unwrap_or(100);

        let retry_strategy = ExponentialBackoff::from_millis(initial_backoff)
            .map(jitter)
            .take(max_attempts);

        let response = Retry::spawn(retry_strategy, || {
            let rb = request_builder.try_clone().ok_or_else(|| {
                Error::Internal("Failed to clone request builder for retry".to_string())
            });
            async move {
                let rb = rb?;
                let response = rb.send().await.map_err(Error::from)?;

                if response.status().is_server_error() {
                    let status = response.status();
                    if status == reqwest::StatusCode::INTERNAL_SERVER_ERROR
                        || status == reqwest::StatusCode::BAD_GATEWAY
                        || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                        || status == reqwest::StatusCode::GATEWAY_TIMEOUT
                    {
                        return Err(Error::from(response.error_for_status().unwrap_err()));
                    }
                }

                Ok(response)
            }
        })
        .await?;

        response.error_for_status().map_err(Error::from)
    }

    /// Returns the system status.
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        tracing::debug!("Fetching SyncThing system status");
        let url = format!("{}/rest/system/status", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<SystemStatus>().await?)
    }

    /// Returns the discovery status.
    pub async fn get_discovery_status(&self) -> Result<DiscoveryResponse> {
        tracing::debug!("Fetching SyncThing discovery status");
        let url = format!("{}/rest/system/discovery", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<DiscoveryResponse>().await?)
    }

    /// Returns the system version.
    pub async fn get_system_version(&self) -> Result<SystemVersion> {
        tracing::debug!("Fetching SyncThing system version");
        let url = format!("{}/rest/system/version", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<SystemVersion>().await?)
    }

    /// Checks for a newer version of Syncthing.
    pub async fn check_upgrade(&self) -> Result<UpgradeResponse> {
        tracing::debug!("Checking for Syncthing upgrade");
        let url = format!("{}/rest/system/upgrade", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = match self.send_with_retry(request).await {
            Ok(r) => r,
            Err(e) => {
                if let crate::error::Error::Api(ref re) = e
                    && re.status() == Some(reqwest::StatusCode::NOT_IMPLEMENTED)
                {
                    return Err(crate::error::Error::SyncThing(
                        "upgrade unsupported".to_string(),
                    ));
                }
                return Err(e);
            }
        };

        let text: String = response.text().await?;

        if text.trim() == "upgrade unsupported" {
            return Err(crate::error::Error::SyncThing(
                "upgrade unsupported".to_string(),
            ));
        }

        Ok(serde_json::from_str::<UpgradeResponse>(&text)?)
    }

    /// Performs an upgrade to the latest version of Syncthing.
    pub async fn perform_upgrade(&self) -> Result<()> {
        tracing::debug!("Performing Syncthing upgrade");
        let url = format!("{}/rest/system/upgrade", self.config.url);
        let request = self.add_auth(self.client.post(&url));
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Returns the full configuration.
    pub async fn get_config(&self) -> Result<Config> {
        tracing::debug!("Fetching full SyncThing configuration");
        let url = format!("{}/rest/config", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<Config>().await?)
    }

    /// Sets the full configuration.
    pub async fn set_config(&self, config: Config) -> Result<()> {
        tracing::debug!("Setting full SyncThing configuration");
        let url = format!("{}/rest/config", self.config.url);
        let request = self.add_auth(self.client.put(&url)).json(&config);
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Lists all folders.
    pub async fn list_folders(&self) -> Result<Vec<FolderConfig>> {
        tracing::debug!("Listing SyncThing folders");
        let url = format!("{}/rest/config/folders", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<Vec<FolderConfig>>().await?)
    }

    /// Adds a new folder.
    pub async fn add_folder(&self, folder_id: &str, label: &str, path: &str) -> Result<()> {
        tracing::debug!("Adding SyncThing folder: {}", folder_id);
        let url = format!("{}/rest/config/folders", self.config.url);
        let folder = serde_json::json!({
            "id": folder_id,
            "label": label,
            "path": path,
        });
        let request = self.add_auth(self.client.post(&url)).json(&folder);
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Returns the configuration for a specific folder.
    pub async fn get_folder(&self, folder_id: &str) -> Result<FolderConfig> {
        tracing::debug!("Fetching SyncThing folder: {}", folder_id);
        let url = format!("{}/rest/config/folders/{}", self.config.url, folder_id);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<FolderConfig>().await?)
    }

    /// Patches the configuration for a specific folder.
    pub async fn patch_folder(&self, folder_id: &str, patch: serde_json::Value) -> Result<()> {
        tracing::debug!("Patching SyncThing folder: {}", folder_id);
        let url = format!("{}/rest/config/folders/{}", self.config.url, folder_id);
        let request = self.add_auth(self.client.patch(&url)).json(&patch);
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Returns the ignore patterns for a specific folder.
    pub async fn get_ignores(&self, folder_id: &str) -> Result<IgnoreConfig> {
        tracing::debug!(
            "Fetching SyncThing ignore patterns for folder: {}",
            folder_id
        );
        let url = format!("{}/rest/db/ignores", self.config.url);
        let request = self
            .add_auth(self.client.get(&url))
            .query(&[("folder", folder_id)]);
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<IgnoreConfig>().await?)
    }

    /// Sets the ignore patterns for a specific folder.
    pub async fn set_ignores(&self, folder_id: &str, ignores: Vec<String>) -> Result<()> {
        tracing::debug!(
            "Setting SyncThing ignore patterns for folder: {}",
            folder_id
        );
        let url = format!("{}/rest/db/ignores", self.config.url);
        let body = serde_json::json!({
            "ignore": ignores,
        });
        let request = self
            .add_auth(self.client.post(&url))
            .query(&[("folder", folder_id)])
            .json(&body);
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Lists all devices.
    pub async fn list_devices(&self) -> Result<Vec<DeviceConfig>> {
        tracing::debug!("Listing SyncThing devices");
        let url = format!("{}/rest/config/devices", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<Vec<DeviceConfig>>().await?)
    }

    /// Adds a new device.
    pub async fn add_device(&self, device_id: &str, name: Option<&str>) -> Result<()> {
        tracing::debug!("Adding SyncThing device: {}", device_id);
        let url = format!("{}/rest/config/devices", self.config.url);
        let mut device = serde_json::json!({
            "deviceID": device_id,
            "addresses": ["dynamic"],
        });
        if let Some(name) = name {
            device["name"] = serde_json::json!(name);
        }
        let request = self.add_auth(self.client.post(&url)).json(&device);
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Removes a device.
    pub async fn remove_device(&self, device_id: &str) -> Result<()> {
        tracing::debug!("Removing SyncThing device: {}", device_id);
        let url = format!("{}/rest/config/devices/{}", self.config.url, device_id);
        let request = self.add_auth(self.client.delete(&url));
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Patches the configuration for a specific device.
    pub async fn patch_device(&self, device_id: &str, patch: serde_json::Value) -> Result<()> {
        tracing::debug!("Patching SyncThing device: {}", device_id);
        let url = format!("{}/rest/config/devices/{}", self.config.url, device_id);
        let request = self.add_auth(self.client.patch(&url)).json(&patch);
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Validates and formats a device ID.
    pub async fn validate_device_id(&self, device_id: &str) -> Result<DeviceIdResponse> {
        tracing::debug!("Validating SyncThing device ID: {}", device_id);
        let url = format!("{}/rest/svc/deviceid", self.config.url);
        let request = self
            .add_auth(self.client.get(&url))
            .query(&[("id", device_id)]);
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<DeviceIdResponse>().await?)
    }

    /// Returns the status for a specific folder.
    pub async fn get_folder_status(&self, folder_id: &str) -> Result<FolderStatus> {
        tracing::debug!("Fetching SyncThing folder status: {}", folder_id);
        let url = format!("{}/rest/db/status", self.config.url);
        let request = self
            .add_auth(self.client.get(&url))
            .query(&[("folder", folder_id)]);
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<FolderStatus>().await?)
    }

    /// Returns the completion status for a specific device.
    pub async fn get_device_completion(&self, device_id: &str) -> Result<FolderCompletion> {
        tracing::debug!("Fetching SyncThing device completion: {}", device_id);
        let url = format!("{}/rest/db/completion", self.config.url);
        let request = self
            .add_auth(self.client.get(&url))
            .query(&[("device", device_id)]);
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<FolderCompletion>().await?)
    }

    /// Triggers a rescan of a folder, or all folders if None is provided.
    pub async fn rescan(&self, folder_id: Option<&str>) -> Result<()> {
        tracing::debug!("Triggering rescan (folder: {:?})", folder_id);
        let url = format!("{}/rest/db/scan", self.config.url);
        let mut request_builder = self.add_auth(self.client.post(&url));
        if let Some(id) = folder_id {
            request_builder = request_builder.query(&[("folder", id)]);
        }
        self.send_with_retry(request_builder).await?;
        Ok(())
    }

    /// Reverts local changes in a Receive Only folder.
    pub async fn revert_folder(&self, folder_id: &str) -> Result<()> {
        tracing::debug!("Reverting folder: {}", folder_id);
        let url = format!("{}/rest/db/revert", self.config.url);
        let request = self
            .add_auth(self.client.post(&url))
            .query(&[("folder", folder_id)]);
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Restarts SyncThing.
    pub async fn restart(&self) -> Result<()> {
        tracing::debug!("Triggering SyncThing restart");
        let url = format!("{}/rest/system/restart", self.config.url);
        let request = self.add_auth(self.client.post(&url));
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Shuts down SyncThing.
    pub async fn shutdown(&self) -> Result<()> {
        tracing::debug!("Triggering SyncThing shutdown");
        let url = format!("{}/rest/system/shutdown", self.config.url);
        let request = self.add_auth(self.client.post(&url));
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Returns whether the running configuration is in sync with the on-disk configuration.
    pub async fn is_config_insync(&self) -> Result<ConfigInSync> {
        tracing::debug!("Checking if SyncThing configuration is in sync");
        let url = format!("{}/rest/system/config/insync", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<ConfigInSync>().await?)
    }

    /// Returns the current list of active system GUI errors.
    pub async fn get_errors(&self) -> Result<SystemErrors> {
        tracing::debug!("Fetching SyncThing system errors");
        let url = format!("{}/rest/system/error", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<SystemErrors>().await?)
    }

    /// Clears SyncThing errors.
    pub async fn clear_errors(&self) -> Result<()> {
        tracing::debug!("Clearing SyncThing errors");
        let url = format!("{}/rest/system/error/clear", self.config.url);
        let request = self.add_auth(self.client.post(&url));
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Returns the last events.
    pub async fn get_events(&self, since: Option<u64>, limit: Option<u32>) -> Result<Vec<Event>> {
        tracing::debug!(
            "Fetching SyncThing events (since: {:?}, limit: {:?})",
            since,
            limit
        );
        let url = format!("{}/rest/events", self.config.url);
        let mut request = self.add_auth(self.client.get(&url));

        let mut query = Vec::new();
        if let Some(s) = since {
            query.push(("since", s.to_string()));
        }
        if let Some(l) = limit {
            query.push(("limit", l.to_string()));
        }

        if !query.is_empty() {
            request = request.query(&query);
        }

        let response = self.send_with_retry(request).await?;
        Ok(response.json::<Vec<Event>>().await?)
    }

    /// Browses a folder.
    pub async fn browse(
        &self,
        folder: &str,
        prefix: Option<&str>,
        levels: Option<u32>,
    ) -> Result<serde_json::Value> {
        tracing::debug!(
            "Browsing SyncThing folder: {} (prefix: {:?}, levels: {:?})",
            folder,
            prefix,
            levels
        );
        let url = format!("{}/rest/db/browse", self.config.url);
        let mut request = self.add_auth(self.client.get(&url));

        let mut query = vec![("folder", folder.to_string())];
        if let Some(p) = prefix {
            query.push(("prefix", p.to_string()));
        }
        if let Some(l) = levels {
            query.push(("levels", l.to_string()));
        }

        request = request.query(&query);

        let response = self.send_with_retry(request).await?;
        Ok(response.json::<serde_json::Value>().await?)
    }

    /// Returns pending devices.
    pub async fn get_pending_devices(&self) -> Result<HashMap<String, PendingDevice>> {
        tracing::debug!("Fetching pending SyncThing devices");
        let url = format!("{}/rest/cluster/pending/devices", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<HashMap<String, PendingDevice>>().await?)
    }

    /// Removes a pending device.
    pub async fn remove_pending_device(&self, device_id: &str) -> Result<()> {
        tracing::debug!("Removing pending SyncThing device: {}", device_id);
        let url = format!("{}/rest/cluster/pending/devices", self.config.url);
        let request = self
            .add_auth(self.client.delete(&url))
            .query(&[("device", device_id)]);
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Returns pending folders.
    pub async fn get_pending_folders(&self) -> Result<HashMap<String, PendingFolder>> {
        tracing::debug!("Fetching pending SyncThing folders");
        let url = format!("{}/rest/cluster/pending/folders", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<HashMap<String, PendingFolder>>().await?)
    }

    /// Removes a pending folder.
    pub async fn remove_pending_folder(&self, folder_id: &str) -> Result<()> {
        tracing::debug!("Removing pending SyncThing folder: {}", folder_id);
        let url = format!("{}/rest/cluster/pending/folders", self.config.url);
        let request = self
            .add_auth(self.client.delete(&url))
            .query(&[("folder", folder_id)]);
        self.send_with_retry(request).await?;
        Ok(())
    }

    /// Returns the connection status for all devices.
    pub async fn get_connections(&self) -> Result<ConnectionsResponse> {
        tracing::debug!("Fetching SyncThing connection status");
        let url = format!("{}/rest/system/connections", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<ConnectionsResponse>().await?)
    }

    /// Returns the system log.
    pub async fn get_system_log(&self) -> Result<SystemLog> {
        tracing::debug!("Fetching SyncThing system log");
        let url = format!("{}/rest/system/log", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<SystemLog>().await?)
    }

    /// Returns device statistics.
    pub async fn get_device_stats(&self) -> Result<HashMap<String, DeviceStats>> {
        tracing::debug!("Fetching SyncThing device statistics");
        let url = format!("{}/rest/stats/device", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<HashMap<String, DeviceStats>>().await?)
    }

    /// Returns folder statistics.
    pub async fn get_folder_stats(&self) -> Result<HashMap<String, FolderStats>> {
        tracing::debug!("Fetching SyncThing folder statistics");
        let url = format!("{}/rest/stats/folder", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<HashMap<String, FolderStats>>().await?)
    }

    /// Returns a ping response.
    pub async fn ping(&self) -> Result<PingResponse> {
        tracing::debug!("Pinging SyncThing instance");
        let url = format!("{}/rest/system/ping", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<PingResponse>().await?)
    }

    /// Returns detailed information about a specific file.
    pub async fn get_file_info(
        &self,
        folder_id: &str,
        file_path: &str,
    ) -> Result<FileInfoResponse> {
        tracing::debug!("Fetching SyncThing file info: {}/{}", folder_id, file_path);
        let url = format!("{}/rest/db/file", self.config.url);
        let request = self
            .add_auth(self.client.get(&url))
            .query(&[("folder", folder_id), ("file", file_path)]);
        let response = self.send_with_retry(request).await?;
        Ok(response.json::<FileInfoResponse>().await?)
    }

    /// Returns the list of files needed to bring a folder up to date.
    pub async fn get_folder_needs(
        &self,
        folder_id: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<FolderNeedResponse> {
        tracing::debug!("Fetching SyncThing folder needs: {}", folder_id);
        let url = format!("{}/rest/db/need", self.config.url);
        let mut request = self
            .add_auth(self.client.get(&url))
            .query(&[("folder", folder_id)]);

        if let Some(p) = page {
            request = request.query(&[("page", p)]);
        }
        if let Some(pp) = per_page {
            request = request.query(&[("perpage", pp)]);
        }

        let response = self.send_with_retry(request).await?;
        Ok(response.json::<FolderNeedResponse>().await?)
    }

    /// Performs a health check on the SyncThing instance.
    pub async fn health_check(&self) -> Result<HealthCheck> {
        tracing::debug!(
            "Performing health check for SyncThing instance: {}",
            self.config.url
        );
        let start = std::time::Instant::now();
        let version_res = self.get_system_version().await;
        let status_res = self.get_system_status().await;
        let insync_res = self.is_config_insync().await;

        let latency = start.elapsed().as_millis();

        match (version_res, status_res, insync_res) {
            (Ok(version), Ok(status), Ok(insync)) => Ok(HealthCheck {
                status: "Online".to_string(),
                latency_ms: latency,
                version: Some(version.version),
                uptime: Some(status.uptime),
                memory_alloc: Some(status.alloc),
                memory_sys: Some(status.total_memory),
                config_insync: Some(insync.insync),
                error: None,
            }),
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => Ok(HealthCheck {
                status: "Offline".to_string(),
                latency_ms: latency,
                version: None,
                uptime: None,
                memory_alloc: None,
                memory_sys: None,
                config_insync: None,
                error: Some(e.to_string()),
            }),
        }
    }
}
