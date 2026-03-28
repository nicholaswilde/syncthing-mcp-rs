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

    /// Masks sensitive information in the configuration.
    ///
    /// This currently masks:
    /// - GUI user, password, and apiKey
    /// - LDAP password
    pub fn mask_sensitive(&mut self) {
        // Mask GUI sensitive fields
        if let Some(gui) = self.config.gui.as_object_mut() {
            if gui.contains_key("user") {
                gui.insert("user".to_string(), serde_json::Value::String("********".to_string()));
            }
            if gui.contains_key("password") {
                gui.insert("password".to_string(), serde_json::Value::String("********".to_string()));
            }
            if gui.contains_key("apiKey") {
                gui.insert("apiKey".to_string(), serde_json::Value::String("********".to_string()));
            }
        }

        // Mask LDAP sensitive fields
        if let Some(ldap) = self.config.ldap.as_object_mut() {
            if ldap.contains_key("password") {
                ldap.insert("password".to_string(), serde_json::Value::String("********".to_string()));
            }
        }
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
