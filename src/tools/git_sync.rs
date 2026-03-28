//! Git-Sync tools for version control of SyncThing configurations.

use crate::api::models::Config;
use crate::error::Result;

/// Exporter for SyncThing configurations to diffable formats.
pub struct ConfigExporter {
    config: Config,
}

impl ConfigExporter {
    /// Creates a new configuration exporter.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Exports the configuration to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.config)?)
    }

    /// Exports the configuration to a YAML string.
    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml_ng::to_string(&self.config)?)
    }
}
