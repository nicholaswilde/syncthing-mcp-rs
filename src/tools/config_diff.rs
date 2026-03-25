//! Configuration diff generator for SyncThing.

use crate::api::models::{Config, DeviceConfig, FolderConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents a difference between two configurations.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConfigDiff {
    /// Folders present in source but missing in destination.
    pub folders_added: Vec<FolderConfig>,
    /// IDs of folders present in destination but missing in source.
    pub folders_removed: Vec<String>,
    /// Folders present in both but requiring update.
    pub folders_updated: Vec<FolderConfig>,
    /// Devices present in source but missing in destination.
    pub devices_added: Vec<DeviceConfig>,
    /// IDs of devices present in destination but missing in source.
    pub devices_removed: Vec<String>,
    /// Devices present in both but requiring update.
    pub devices_updated: Vec<DeviceConfig>,
}

/// Represents a set of changes to be applied to a configuration.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ConfigPatch {
    /// Folders to add or update.
    pub folders: Vec<FolderConfig>,
    /// Devices to add or update.
    pub devices: Vec<DeviceConfig>,
    /// IDs of folders to remove.
    pub folders_to_remove: Vec<String>,
    /// IDs of device IDs to remove.
    pub devices_to_remove: Vec<String>,
}

impl ConfigDiff {
    /// Generates a diff between source and destination configurations.
    pub fn generate(source: &Config, dest: &Config) -> Self {
        let source_folders: std::collections::HashMap<_, _> =
            source.folders.iter().map(|f| (f.id.as_str(), f)).collect();
        let dest_folders: std::collections::HashMap<_, _> =
            dest.folders.iter().map(|f| (f.id.as_str(), f)).collect();

        let source_folder_ids: HashSet<_> = source_folders.keys().cloned().collect();
        let dest_folder_ids: HashSet<_> = dest_folders.keys().cloned().collect();

        let source_devices: std::collections::HashMap<_, _> = source
            .devices
            .iter()
            .map(|d| (d.device_id.as_str(), d))
            .collect();
        let dest_devices: std::collections::HashMap<_, _> = dest
            .devices
            .iter()
            .map(|d| (d.device_id.as_str(), d))
            .collect();

        let source_device_ids: HashSet<_> = source_devices.keys().cloned().collect();
        let dest_device_ids: HashSet<_> = dest_devices.keys().cloned().collect();

        Self {
            folders_added: source_folder_ids
                .difference(&dest_folder_ids)
                .map(|id| (*source_folders.get(id).unwrap()).clone())
                .collect(),
            folders_removed: dest_folder_ids
                .difference(&source_folder_ids)
                .map(|s| s.to_string())
                .collect(),
            folders_updated: source_folder_ids
                .intersection(&dest_folder_ids)
                .filter(|id| {
                    let s = source_folders.get(*id).unwrap();
                    let d = dest_folders.get(*id).unwrap();
                    // Simple equality check for now.
                    // In a real scenario, we might want a deep comparison or skip some fields.
                    serde_json::to_value(s).unwrap() != serde_json::to_value(d).unwrap()
                })
                .map(|id| (*source_folders.get(id).unwrap()).clone())
                .collect(),
            devices_added: source_device_ids
                .difference(&dest_device_ids)
                .map(|id| (*source_devices.get(id).unwrap()).clone())
                .collect(),
            devices_removed: dest_device_ids
                .difference(&source_device_ids)
                .map(|s| s.to_string())
                .collect(),
            devices_updated: source_device_ids
                .intersection(&dest_device_ids)
                .filter(|id| {
                    let s = source_devices.get(*id).unwrap();
                    let d = dest_devices.get(*id).unwrap();
                    serde_json::to_value(s).unwrap() != serde_json::to_value(d).unwrap()
                })
                .map(|id| (*source_devices.get(id).unwrap()).clone())
                .collect(),
        }
    }

    /// Returns a human-readable summary of the diff.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        if !self.folders_removed.is_empty() || !self.devices_removed.is_empty() {
            lines.push(format!(
                "⚠️ WARNING: This action will REMOVE {} folder(s) and {} device(s) from the destination instance.",
                self.folders_removed.len(),
                self.devices_removed.len()
            ));
            lines.push("".to_string());
        }

        if !self.folders_added.is_empty()
            || !self.folders_removed.is_empty()
            || !self.folders_updated.is_empty()
        {
            lines.push(format!(
                "Folders: {} added, {} removed, {} updated.",
                self.folders_added.len(),
                self.folders_removed.len(),
                self.folders_updated.len()
            ));
            let mut added: Vec<_> = self.folders_added.iter().map(|f| &f.id).collect();
            added.sort();
            for id in added {
                lines.push(format!("  + Folder: {}", id));
            }
            let mut removed = self.folders_removed.clone();
            removed.sort();
            for id in removed {
                lines.push(format!("  - Folder: {}", id));
            }
            let mut updated: Vec<_> = self.folders_updated.iter().map(|f| &f.id).collect();
            updated.sort();
            for id in updated {
                lines.push(format!("  ~ Folder: {}", id));
            }
        }

        if !self.devices_added.is_empty()
            || !self.devices_removed.is_empty()
            || !self.devices_updated.is_empty()
        {
            lines.push(format!(
                "Devices: {} added, {} removed, {} updated.",
                self.devices_added.len(),
                self.devices_removed.len(),
                self.devices_updated.len()
            ));
            let mut added: Vec<_> = self.devices_added.iter().map(|d| &d.device_id).collect();
            added.sort();
            for id in added {
                lines.push(format!("  + Device: {}", id));
            }
            let mut removed = self.devices_removed.clone();
            removed.sort();
            for id in removed {
                lines.push(format!("  - Device: {}", id));
            }
            let mut updated: Vec<_> = self.devices_updated.iter().map(|d| &d.device_id).collect();
            updated.sort();
            for id in updated {
                lines.push(format!("  ~ Device: {}", id));
            }
        }

        if lines.is_empty() {
            "No configuration changes detected.".to_string()
        } else {
            lines.join("\n")
        }
    }

    /// Converts the diff into a full patch.
    pub fn to_patch(&self) -> ConfigPatch {
        ConfigPatch {
            folders: self
                .folders_added
                .iter()
                .cloned()
                .chain(self.folders_updated.iter().cloned())
                .collect(),
            devices: self
                .devices_added
                .iter()
                .cloned()
                .chain(self.devices_updated.iter().cloned())
                .collect(),
            folders_to_remove: self.folders_removed.clone(),
            devices_to_remove: self.devices_removed.clone(),
        }
    }
}

/// Calculates the difference between two configurations.
pub fn calculate_diff(base: &Config, head: &Config) -> ConfigDiff {
    ConfigDiff::generate(head, base)
}

/// Applies a patch to a configuration.
pub fn apply_patch(config: &mut Config, patch: &ConfigPatch) -> crate::error::Result<()> {
    // 1. Remove folders
    config
        .folders
        .retain(|f| !patch.folders_to_remove.contains(&f.id));

    // 2. Remove devices
    config
        .devices
        .retain(|d| !patch.devices_to_remove.contains(&d.device_id));

    // 3. Add or update folders
    for new_folder in &patch.folders {
        if let Some(existing) = config.folders.iter_mut().find(|f| f.id == new_folder.id) {
            *existing = new_folder.clone();
        } else {
            config.folders.push(new_folder.clone());
        }
    }

    // 4. Add or update devices
    for new_device in &patch.devices {
        if let Some(existing) = config
            .devices
            .iter_mut()
            .find(|d| d.device_id == new_device.device_id)
        {
            *existing = new_device.clone();
        } else {
            config.devices.push(new_device.clone());
        }
    }

    Ok(())
}
