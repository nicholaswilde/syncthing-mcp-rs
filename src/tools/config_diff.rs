//! Configuration diff generator for SyncThing.

use serde_json::Value;
use std::collections::HashSet;

/// Represents a difference between two configurations.
#[derive(Debug, PartialEq)]
pub struct ConfigDiff {
    /// IDs of folders present in source but missing in destination.
    pub folders_added: Vec<String>,
    /// IDs of folders present in destination but missing in source.
    pub folders_removed: Vec<String>,
    /// IDs of folders present in both but potentially requiring update.
    pub folders_updated: Vec<String>,
    /// IDs of devices present in source but missing in destination.
    pub devices_added: Vec<String>,
    /// IDs of devices present in destination but missing in source.
    pub devices_removed: Vec<String>,
    /// IDs of devices present in both but potentially requiring update.
    pub devices_updated: Vec<String>,
}

impl ConfigDiff {
    /// Generates a diff between source and destination configurations.
    pub fn generate(source: &Value, dest: &Value) -> Self {
        let empty_vec = Vec::new();

        // Extract folder IDs
        let source_folders = source.get("folders").and_then(|f| f.as_array()).unwrap_or(&empty_vec);
        let dest_folders = dest.get("folders").and_then(|f| f.as_array()).unwrap_or(&empty_vec);

        let source_folder_ids: HashSet<_> = source_folders.iter().filter_map(|f| f.get("id").and_then(|id| id.as_str())).collect();
        let dest_folder_ids: HashSet<_> = dest_folders.iter().filter_map(|f| f.get("id").and_then(|id| id.as_str())).collect();

        // Extract device IDs
        let source_devices = source.get("devices").and_then(|d| d.as_array()).unwrap_or(&empty_vec);
        let dest_devices = dest.get("devices").and_then(|d| d.as_array()).unwrap_or(&empty_vec);

        let source_device_ids: HashSet<_> = source_devices.iter().filter_map(|d| d.get("deviceID").and_then(|id| id.as_str())).collect();
        let dest_device_ids: HashSet<_> = dest_devices.iter().filter_map(|d| d.get("deviceID").and_then(|id| id.as_str())).collect();

        Self {
            folders_added: source_folder_ids.difference(&dest_folder_ids).map(|s| s.to_string()).collect(),
            folders_removed: dest_folder_ids.difference(&source_folder_ids).map(|s| s.to_string()).collect(),
            folders_updated: source_folder_ids.intersection(&dest_folder_ids).map(|s| s.to_string()).collect(),
            devices_added: source_device_ids.difference(&dest_device_ids).map(|s| s.to_string()).collect(),
            devices_removed: dest_device_ids.difference(&source_device_ids).map(|s| s.to_string()).collect(),
            devices_updated: source_device_ids.intersection(&dest_device_ids).map(|s| s.to_string()).collect(),
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
            let mut added = self.folders_added.clone();
            added.sort();
            for id in added {
                lines.push(format!("  + Folder: {}", id));
            }
            let mut removed = self.folders_removed.clone();
            removed.sort();
            for id in removed {
                lines.push(format!("  - Folder: {}", id));
            }
            let mut updated = self.folders_updated.clone();
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
            let mut added = self.devices_added.clone();
            added.sort();
            for id in added {
                lines.push(format!("  + Device: {}", id));
            }
            let mut removed = self.devices_removed.clone();
            removed.sort();
            for id in removed {
                lines.push(format!("  - Device: {}", id));
            }
            let mut updated = self.devices_updated.clone();
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
}
