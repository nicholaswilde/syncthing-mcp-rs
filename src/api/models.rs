use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemStatus {
    #[serde(rename = "myID")]
    pub my_id: String,
    pub uptime: u64,
    pub alloc: u64,
    #[serde(rename = "sys")]
    pub total_memory: u64,
    pub goroutines: u32,
    #[serde(rename = "pathSeparator")]
    pub path_separator: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemVersion {
    pub version: String,
    pub arch: String,
    pub os: String,
    #[serde(rename = "isRelease")]
    pub is_release: bool,
    #[serde(rename = "isBeta")]
    pub is_beta: bool,
    #[serde(rename = "isCandidate")]
    pub is_candidate: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderConfig {
    pub id: String,
    pub label: String,
    pub path: String,
    #[serde(rename = "type")]
    pub folder_type: String,
    pub devices: Vec<FolderDeviceConfiguration>,
    #[serde(default)]
    pub rescan_interval_s: u32,
    #[serde(default)]
    pub fs_watcher_enabled: bool,
    #[serde(default)]
    pub paused: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderDeviceConfiguration {
    #[serde(rename = "deviceID")]
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceConfig {
    #[serde(rename = "deviceID")]
    pub device_id: String,
    pub name: Option<String>,
    pub addresses: Vec<String>,
    #[serde(default)]
    pub compression: String,
    #[serde(default)]
    pub introducer: bool,
    #[serde(default)]
    pub paused: bool,
    #[serde(default)]
    pub untrusted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgnoreConfig {
    pub ignore: Option<Vec<String>>,
    pub expanded: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderStatus {
    pub state: String,
    #[serde(rename = "needBytes", default)]
    pub need_bytes: u64,
    #[serde(rename = "needFiles", default)]
    pub need_files: u64,
    #[serde(rename = "inSyncBytes", default)]
    pub in_sync_bytes: u64,
    #[serde(rename = "inSyncFiles", default)]
    pub in_sync_files: u64,
    #[serde(rename = "globalBytes", default)]
    pub global_bytes: u64,
    #[serde(rename = "globalFiles", default)]
    pub global_files: u64,
    #[serde(rename = "localBytes", default)]
    pub local_bytes: u64,
    #[serde(rename = "localFiles", default)]
    pub local_files: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceCompletion {
    #[serde(default)]
    pub completion: f64,
    #[serde(rename = "needBytes", default)]
    pub need_bytes: u64,
    #[serde(rename = "needFiles", default)]
    pub need_files: u64,
    #[serde(rename = "globalBytes", default)]
    pub global_bytes: u64,
}
