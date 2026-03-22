use crate::api::models::*;
use crate::config::InstanceConfig;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct SyncThingClient {
    pub client: reqwest::Client,
    pub config: InstanceConfig,
}

impl SyncThingClient {
    pub fn new(config: InstanceConfig) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.no_verify_ssl.unwrap_or(true))
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

    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        tracing::debug!("Fetching SyncThing system status");
        let url = format!("{}/rest/system/status", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<SystemStatus>().await?)
    }

    pub async fn get_system_version(&self) -> Result<SystemVersion> {
        tracing::debug!("Fetching SyncThing system version");
        let url = format!("{}/rest/system/version", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<SystemVersion>().await?)
    }

    pub async fn list_folders(&self) -> Result<Vec<FolderConfig>> {
        tracing::debug!("Listing SyncThing folders");
        let url = format!("{}/rest/config/folders", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<Vec<FolderConfig>>().await?)
    }

    pub async fn add_folder(&self, folder_id: &str, label: &str, path: &str) -> Result<()> {
        tracing::debug!("Adding SyncThing folder: {}", folder_id);
        let url = format!("{}/rest/config/folders", self.config.url);
        let folder = serde_json::json!({
            "id": folder_id,
            "label": label,
            "path": path,
        });
        let request = self.add_auth(self.client.post(&url)).json(&folder);
        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_folder(&self, folder_id: &str) -> Result<FolderConfig> {
        tracing::debug!("Fetching SyncThing folder: {}", folder_id);
        let url = format!("{}/rest/config/folders/{}", self.config.url, folder_id);
        let request = self.add_auth(self.client.get(&url));
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<FolderConfig>().await?)
    }

    pub async fn patch_folder(&self, folder_id: &str, patch: serde_json::Value) -> Result<()> {
        tracing::debug!("Patching SyncThing folder: {}", folder_id);
        let url = format!("{}/rest/config/folders/{}", self.config.url, folder_id);
        let request = self.add_auth(self.client.patch(&url)).json(&patch);
        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_ignores(&self, folder_id: &str) -> Result<IgnoreConfig> {
        tracing::debug!(
            "Fetching SyncThing ignore patterns for folder: {}",
            folder_id
        );
        let url = format!("{}/rest/db/ignores", self.config.url);
        let request = self
            .add_auth(self.client.get(&url))
            .query(&[("folder", folder_id)]);
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<IgnoreConfig>().await?)
    }

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
        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn list_devices(&self) -> Result<Vec<DeviceConfig>> {
        tracing::debug!("Listing SyncThing devices");
        let url = format!("{}/rest/config/devices", self.config.url);
        let request = self.add_auth(self.client.get(&url));
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<Vec<DeviceConfig>>().await?)
    }

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
        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn remove_device(&self, device_id: &str) -> Result<()> {
        tracing::debug!("Removing SyncThing device: {}", device_id);
        let url = format!("{}/rest/config/devices/{}", self.config.url, device_id);
        let request = self.add_auth(self.client.delete(&url));
        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn patch_device(&self, device_id: &str, patch: serde_json::Value) -> Result<()> {
        tracing::debug!("Patching SyncThing device: {}", device_id);
        let url = format!("{}/rest/config/devices/{}", self.config.url, device_id);
        let request = self.add_auth(self.client.patch(&url)).json(&patch);
        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_folder_status(&self, folder_id: &str) -> Result<FolderStatus> {
        tracing::debug!("Fetching SyncThing folder status: {}", folder_id);
        let url = format!("{}/rest/db/status", self.config.url);
        let request = self
            .add_auth(self.client.get(&url))
            .query(&[("folder", folder_id)]);
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<FolderStatus>().await?)
    }

    pub async fn get_device_completion(&self, device_id: &str) -> Result<DeviceCompletion> {
        tracing::debug!("Fetching SyncThing device completion: {}", device_id);
        let url = format!("{}/rest/db/completion", self.config.url);
        let request = self
            .add_auth(self.client.get(&url))
            .query(&[("device", device_id)]);
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<DeviceCompletion>().await?)
    }

    pub async fn rescan(&self, folder_id: Option<&str>) -> Result<()> {
        tracing::debug!("Triggering rescan (folder: {:?})", folder_id);
        let url = format!("{}/rest/db/scan", self.config.url);
        let mut request = self.add_auth(self.client.post(&url));
        if let Some(id) = folder_id {
            request = request.query(&[("folder", id)]);
        }
        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn restart(&self) -> Result<()> {
        tracing::debug!("Triggering SyncThing restart");
        let url = format!("{}/rest/system/restart", self.config.url);
        let request = self.add_auth(self.client.post(&url));
        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn clear_errors(&self) -> Result<()> {
        tracing::debug!("Clearing SyncThing errors");
        let url = format!("{}/rest/system/error/clear", self.config.url);
        let request = self.add_auth(self.client.post(&url));
        request.send().await?.error_for_status()?;
        Ok(())
    }
}
