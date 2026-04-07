use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System status information.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FolderDeviceConfiguration {
    /// The device identifier.
    #[serde(rename = "deviceID")]
    pub device_id: String,
}

/// Device configuration.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct IgnoreConfig {
    /// The list of ignore patterns.
    pub ignore: Option<Vec<String>>,
    /// The list of expanded ignore patterns.
    pub expanded: Option<Vec<String>>,
}

/// Folder status information.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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

/// Folder or device completion status.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FolderCompletion {
    /// Completion percentage (0.0 to 100.0).
    pub completion: f64,
    /// Total size of the folder in bytes (global state).
    #[serde(rename = "globalBytes")]
    pub global_bytes: u64,
    /// Number of bytes still needed to reach 100% completion.
    #[serde(rename = "needBytes")]
    pub need_bytes: u64,
    /// Total number of files, directories, and symlinks in the folder.
    #[serde(rename = "globalItems")]
    pub global_items: u64,
    /// Number of items still needed to be downloaded/updated.
    #[serde(rename = "needItems")]
    pub need_items: u64,
    /// Number of items that need to be deleted.
    #[serde(rename = "needDeletes")]
    pub need_deletes: u64,
    /// State of the remote device.
    #[serde(rename = "remoteState", default)]
    pub remote_state: String,
    /// Current internal sequence number.
    #[serde(default)]
    pub sequence: u64,
}

/// An event from the SyncThing event API.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Event {
    /// The event ID.
    pub id: u64,
    /// The type of event.
    #[serde(rename = "type")]
    pub event_type: String,
    /// The time the event occurred.
    pub time: String,
    /// Data associated with the event.
    pub data: Option<EventData>,
}

impl Event {
    /// Returns a human-readable summary of the event.
    pub fn summary(&self) -> String {
        match &self.data {
            Some(EventData::FolderStateChanged {
                folder, from, to, ..
            }) => {
                format!("Folder '{}' changed state from {} to {}", folder, from, to)
            }
            Some(EventData::DeviceConnected {
                device,
                addr,
                conn_type,
            }) => {
                format!(
                    "Device '{}' connected via {} at {}",
                    device, conn_type, addr
                )
            }
            Some(EventData::DeviceDisconnected { device, error }) => {
                format!("Device '{}' disconnected: {}", device, error)
            }
            Some(EventData::LocalIndexUpdated { folder, filenames }) => {
                format!(
                    "Local index updated for folder '{}' ({} files)",
                    folder,
                    filenames.len()
                )
            }
            _ => format!("Event: {}", self.event_type),
        }
    }
}

/// Heterogeneous data associated with SyncThing events.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum EventData {
    /// Data for FolderStateChanged event.
    FolderStateChanged {
        /// The folder ID.
        folder: String,
        /// From state.
        from: String,
        /// To state.
        to: String,
        /// Error message if any.
        error: Option<String>,
    },
    /// Data for DeviceConnected event.
    DeviceConnected {
        /// The device ID.
        device: String,
        /// The address.
        addr: String,
        /// The type of connection.
        #[serde(rename = "type")]
        conn_type: String,
    },
    /// Data for DeviceDisconnected event.
    DeviceDisconnected {
        /// The device ID.
        device: String,
        /// Error message if any.
        error: String,
    },
    /// Data for LocalIndexUpdated event.
    LocalIndexUpdated {
        /// The folder ID.
        folder: String,
        /// The file name.
        filenames: Vec<String>,
    },
    /// Generic data for other events.
    Generic(serde_json::Value),
}

/// A device that is pending acceptance.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PendingDevice {
    /// The time the device was last seen.
    pub time: String,
    /// The name of the device.
    pub name: String,
    /// The address of the device.
    pub address: String,
}

/// Connection status for a device.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ConnectionStatus {
    /// The time the status was recorded.
    pub at: Option<String>,
    /// Total bytes received.
    #[serde(rename = "inBytesTotal", default)]
    pub in_bytes_total: u64,
    /// Total bytes sent.
    #[serde(rename = "outBytesTotal", default)]
    pub out_bytes_total: u64,
    /// The remote address.
    pub address: Option<String>,
    /// The client version of the remote device.
    #[serde(rename = "clientVersion")]
    pub client_version: Option<String>,
    /// Whether the device is currently connected.
    pub connected: bool,
    /// The type of connection.
    #[serde(rename = "type")]
    pub connection_type: Option<String>,
    /// Whether the connection is paused.
    #[serde(default)]
    pub paused: bool,
    /// Cryptographic suite used for the connection.
    pub crypto: Option<String>,
    /// Whether the connection is local (vs remote/relay).
    #[serde(rename = "isLocal")]
    pub is_local: Option<bool>,
    /// The MAC address (if available on local network).
    pub mac: Option<String>,
}

/// Total connection statistics.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ConnectionsTotal {
    /// Total bytes received across all connections.
    #[serde(rename = "inBytesTotal", default)]
    pub in_bytes_total: u64,
    /// Total bytes sent across all connections.
    #[serde(rename = "outBytesTotal", default)]
    pub out_bytes_total: u64,
}

/// Response from /rest/system/connections.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ConnectionsResponse {
    /// Map of device IDs to connection status.
    pub connections: HashMap<String, ConnectionStatus>,
    /// Total connection statistics.
    #[serde(default)]
    pub total: ConnectionsTotal,
}

/// A single log entry from Syncthing.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct LogEntry {
    /// The timestamp of the log entry.
    pub when: String,
    /// The log message.
    pub message: String,
}

/// Response from /rest/system/log.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct SystemLog {
    /// The list of log entries.
    pub messages: Vec<LogEntry>,
}

/// Response from /rest/system/error.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct SystemErrors {
    /// The list of system errors.
    pub errors: Option<Vec<LogEntry>>,
}

/// Statistics for a device.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct DeviceStats {
    /// The time the device was last seen.
    #[serde(rename = "lastSeen")]
    pub last_seen: String,
    /// The duration of the last connection in seconds.
    #[serde(rename = "lastConnectionDurationS")]
    pub last_connection_duration_s: f64,
}

/// Statistics for a folder.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FolderStats {
    /// The time of the last scan.
    #[serde(rename = "lastScan")]
    pub last_scan: String,
    /// Information about the last file synced.
    #[serde(rename = "lastFile")]
    pub last_file: Option<LastFileStats>,
}

/// Information about the last file synced in a folder.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct LastFileStats {
    /// The name of the file.
    pub filename: String,
    /// The time the file was synced.
    pub at: String,
}

/// A folder that is pending acceptance.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PendingFolder {
    /// Devices that have offered this folder.
    #[serde(rename = "offeredBy")]
    pub offered_by: HashMap<String, OfferedBy>,
}

/// Information about a device offering a pending folder.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct OfferedBy {
    /// The time the folder was offered.
    pub time: String,
    /// The suggested label for the folder.
    pub label: String,
    /// Whether the data is received encrypted.
    #[serde(rename = "receiveEncrypted")]
    pub receive_encrypted: bool,
    /// Whether the remote device is encrypted.
    #[serde(rename = "remoteEncrypted")]
    pub remote_encrypted: bool,
}

/// Response from /rest/svc/deviceid.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct DeviceIdResponse {
    /// The formatted device ID.
    pub id: Option<String>,
    /// Error message if the ID is invalid.
    pub error: Option<String>,
}

/// Configuration for the Syncthing GUI/Web UI.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GuiConfig {
    /// Whether the GUI is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// The address the GUI should listen on (e.g., "127.0.0.1:8384").
    #[serde(default)]
    pub address: String,
    /// The username for GUI authentication.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// The hashed password for GUI authentication.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Whether to use HTTPS for the GUI.
    #[serde(rename = "useTLS", default)]
    pub use_tls: bool,
    /// The API key for accessing the REST API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// The active theme (e.g., "default", "dark").
    #[serde(default)]
    pub theme: String,
    /// The debugging/profiling setting.
    #[serde(default)]
    pub debugging: bool,
    /// Whether to allow insecure admin access over HTTP.
    #[serde(default)]
    pub insecure_admin_access: bool,
    /// Whether to skip the host check.
    #[serde(default)]
    pub insecure_skip_hostcheck: bool,
    /// Whether to allow frame authentication (embedding in iframes).
    #[serde(default)]
    pub insecure_allow_frame_auth: bool,
    /// The UI language.
    #[serde(default)]
    pub send_basic_stats: bool,
}

/// The root configuration structure.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Config {
    /// Configuration version.
    pub version: u32,
    /// List of folders.
    pub folders: Vec<FolderConfig>,
    /// List of devices.
    pub devices: Vec<DeviceConfig>,
    /// GUI configuration.
    pub gui: GuiConfig,
    /// LDAP configuration.
    pub ldap: serde_json::Value,
    /// Global options.
    pub options: serde_json::Value,
    /// Remote ignored devices.
    #[serde(rename = "remoteIgnoredDevices")]
    pub remote_ignored_devices: serde_json::Value,
    /// Default configuration.
    pub defaults: serde_json::Value,
}

/// Response from /rest/system/config/insync.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ConfigInSync {
    /// Whether the configuration is in sync.
    #[serde(rename = "configInSync")]
    pub insync: bool,
}

/// Health check result for a SyncThing instance.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct HealthCheck {
    /// The status of the instance (e.g., "Online", "Offline").
    pub status: String,
    /// Response latency in milliseconds.
    pub latency_ms: u128,
    /// The SyncThing version if available.
    pub version: Option<String>,
    /// Uptime in seconds.
    pub uptime: Option<u64>,
    /// Memory allocated in bytes.
    pub memory_alloc: Option<u64>,
    /// Total system memory in bytes.
    pub memory_sys: Option<u64>,
    /// Whether the configuration is in sync with on-disk.
    pub config_insync: Option<bool>,
    /// Error message if any.
    pub error: Option<String>,
}

/// Detailed information about a file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FileInfoResponse {
    /// List of devices having the file.
    pub availability: Vec<FileAvailability>,
    /// Global state of the file.
    pub global: FileMetadata,
    /// Local state of the file.
    pub local: FileMetadata,
    /// Modification time information.
    pub mtime: MtimeInfo,
}

/// Information about a device having a file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FileAvailability {
    /// The device identifier.
    pub id: String,
    /// Whether the file is from a temporary location.
    #[serde(rename = "fromTemporary")]
    pub from_temporary: bool,
}

/// Metadata for a file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FileMetadata {
    /// The file name.
    pub name: String,
    /// The file type.
    #[serde(rename = "type")]
    pub file_type: String,
    /// The file size in bytes.
    pub size: u64,
    /// The file permissions.
    pub permissions: u32,
    /// Modified time in seconds.
    #[serde(rename = "modifiedS")]
    pub modified_s: i64,
    /// Modified time in nanoseconds.
    #[serde(rename = "modifiedNs")]
    pub modified_ns: u32,
    /// The ID of the device that last modified the file.
    #[serde(rename = "modifiedBy")]
    pub modified_by: String,
    /// The file version (vector clock).
    pub version: FileVersion,
    /// The sequence number.
    pub sequence: u64,
    /// The list of file blocks.
    pub blocks: Option<Vec<FileBlock>>,
    /// Whether the file has no permissions.
    #[serde(rename = "noPermissions")]
    pub no_permissions: bool,
    /// Whether the file is invalid.
    pub invalid: bool,
    /// Whether the file is deleted.
    pub deleted: bool,
    /// Whether the file is ignored.
    pub ignored: bool,
    /// Whether the file must be rescanned.
    #[serde(rename = "mustRescan")]
    pub must_rescan: bool,
}

/// Vector clock versioning for a file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FileVersion {
    /// The list of version counters.
    pub counters: Vec<VersionCounter>,
}

/// A single counter in a vector clock.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct VersionCounter {
    /// The device identifier.
    pub id: String,
    /// The counter value.
    pub value: u64,
}

/// A block of a file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FileBlock {
    /// The offset of the block.
    pub offset: i64,
    /// The size of the block.
    pub size: u32,
    /// The hash of the block.
    pub hash: String,
}

/// Modification time information.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct MtimeInfo {
    /// Error message if any.
    pub err: Option<String>,
    /// The modification time values.
    pub value: MtimeValue,
}

/// Actual modification time values.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct MtimeValue {
    /// The real modification time.
    pub real: String,
    /// The virtual modification time.
    #[serde(rename = "virtual")]
    pub virtual_time: String,
}

/// Response from /rest/db/need.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FolderNeedResponse {
    /// Files currently in progress.
    pub progress: Vec<NeedFile>,
    /// Files queued for synchronization.
    pub queued: Vec<NeedFile>,
    /// Other needed files.
    pub rest: Vec<NeedFile>,
    /// Current page number.
    pub page: u32,
    /// Files per page.
    pub perpage: u32,
    /// Total number of needed files.
    pub total: Option<u32>,
}

/// A file that is needed for synchronization.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct NeedFile {
    /// The sequence number.
    pub sequence: u64,
    /// The last modification time.
    pub modified: String,
    /// The file name.
    pub name: String,
    /// The file size in bytes.
    pub size: u64,
    /// The file version.
    pub version: Vec<String>,
    /// The file type.
    #[serde(rename = "type")]
    pub file_type: String,
    /// The file permissions.
    pub permissions: Option<String>,
}

/// Discovery information for a device.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct DiscoveryInfo {
    /// List of announced addresses.
    pub addresses: Vec<String>,
}

/// Response from /rest/system/discovery.
pub type DiscoveryResponse = HashMap<String, DiscoveryInfo>;

/// Upgrade information for the SyncThing instance.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct UpgradeResponse {
    /// The latest available version.
    pub latest: String,
    /// Whether a newer version is available.
    pub newer: bool,
    /// Whether a major newer version is available.
    #[serde(rename = "majorNewer")]
    pub major_newer: bool,
    /// The currently running version.
    pub running: String,
}

/// Response from /rest/system/ping.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PingResponse {
    /// Should be "pong".
    pub ping: String,
}
