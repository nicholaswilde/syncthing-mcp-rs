use serde::{Deserialize, Serialize};

/// System status information.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemStatus {
    /// The unique identifier for this SyncThing instance.
    #[serde(rename = "myID")]
    pub my_id: String,
    /// Uptime in seconds.
    pub uptime: u64,
    /// Memory allocated in bytes.
    pub alloc: u64,
    /// Total system memory in bytes.
    #[serde(rename = "sys")]
    pub total_memory: u64,
    /// Number of Goroutines.
    pub goroutines: u32,
    /// The path separator used by the system.
    #[serde(rename = "pathSeparator")]
    pub path_separator: String,
}

/// System version information.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemVersion {
    /// The version string.
    pub version: String,
    /// The architecture.
    pub arch: String,
    /// The operating system.
    pub os: String,
    /// Whether this is a release version.
    #[serde(rename = "isRelease")]
    pub is_release: bool,
    /// Whether this is a beta version.
    #[serde(rename = "isBeta")]
    pub is_beta: bool,
    /// Whether this is a candidate version.
    #[serde(rename = "isCandidate")]
    pub is_candidate: bool,
}

/// Folder configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderConfig {
    /// The folder identifier.
    pub id: String,
    /// The folder label.
    pub label: String,
    /// The file system path.
    pub path: String,
    /// The folder type (e.g., "sendreceive").
    #[serde(rename = "type")]
    pub folder_type: String,
    /// The devices this folder is shared with.
    pub devices: Vec<FolderDeviceConfiguration>,
    /// Rescan interval in seconds.
    #[serde(default)]
    pub rescan_interval_s: u32,
    /// Whether the file system watcher is enabled.
    #[serde(default)]
    pub fs_watcher_enabled: bool,
    /// Whether the folder is paused.
    #[serde(default)]
    pub paused: bool,
}

/// Configuration for a device associated with a folder.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderDeviceConfiguration {
    /// The device identifier.
    #[serde(rename = "deviceID")]
    pub device_id: String,
}

/// Device configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceConfig {
    /// The device identifier.
    #[serde(rename = "deviceID")]
    pub device_id: String,
    /// The device name.
    pub name: Option<String>,
    /// The device addresses.
    pub addresses: Vec<String>,
    /// Compression setting.
    #[serde(default)]
    pub compression: String,
    /// Whether this device is an introducer.
    #[serde(default)]
    pub introducer: bool,
    /// Whether the device is paused.
    #[serde(default)]
    pub paused: bool,
    /// Whether the device is untrusted.
    #[serde(default)]
    pub untrusted: bool,
}

/// Ignore patterns for a folder.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgnoreConfig {
    /// The list of ignore patterns.
    pub ignore: Option<Vec<String>>,
    /// The list of expanded ignore patterns.
    pub expanded: Option<Vec<String>>,
}

/// Folder status information.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderStatus {
    /// The current state of the folder.
    pub state: String,
    /// Number of bytes needed to be in sync.
    #[serde(rename = "needBytes", default)]
    pub need_bytes: u64,
    /// Number of files needed to be in sync.
    #[serde(rename = "needFiles", default)]
    pub need_files: u64,
    /// Number of bytes currently in sync.
    #[serde(rename = "inSyncBytes", default)]
    pub in_sync_bytes: u64,
    /// Number of files currently in sync.
    #[serde(rename = "inSyncFiles", default)]
    pub in_sync_files: u64,
    /// Global total bytes in the folder.
    #[serde(rename = "globalBytes", default)]
    pub global_bytes: u64,
    /// Global total files in the folder.
    #[serde(rename = "globalFiles", default)]
    pub global_files: u64,
    /// Local total bytes in the folder.
    #[serde(rename = "localBytes", default)]
    pub local_bytes: u64,
    /// Local total files in the folder.
    #[serde(rename = "localFiles", default)]
    pub local_files: u64,
}

/// Device completion status.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceCompletion {
    /// Completion percentage (0.0 to 100.0).
    #[serde(default)]
    pub completion: f64,
    /// Number of bytes needed by the device.
    #[serde(rename = "needBytes", default)]
    pub need_bytes: u64,
    /// Number of files needed by the device.
    #[serde(rename = "needFiles", default)]
    pub need_files: u64,
    /// Global total bytes.
    #[serde(rename = "globalBytes", default)]
    pub global_bytes: u64,
}

/// An event from the SyncThing event API.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    /// The event ID.
    pub id: u64,
    /// The type of event.
    #[serde(rename = "type")]
    pub event_type: String,
    /// The time the event occurred.
    pub time: String,
    /// Optional data associated with the event.
    pub data: Option<serde_json::Value>,
}

/// A device that is pending acceptance.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingDevice {
    /// The time the device was last seen.
    pub time: String,
    /// The name of the device.
    pub name: String,
    /// The address of the device.
    pub address: String,
}
