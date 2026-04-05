/// Bandwidth orchestration tools.
pub mod bandwidth;
/// Unit tests for bandwidth MCP tools.
#[cfg(test)]
mod bandwidth_mcp_tests;
/// Unit tests for bandwidth tools.
#[cfg(test)]
mod bandwidth_tests;
/// Unit tests for the batch_manage_folders tool.
#[cfg(test)]
mod batch_folder_tests;
/// Unit tests for browse_folder optimization.
#[cfg(test)]
mod browse_folder_optimization_tests;
/// Browser tool for exploring synced folders.
pub mod browser;
/// Unit tests for browser tools.
#[cfg(test)]
mod browser_tests;
/// Configuration replication tool.
pub mod config;
/// Configuration diff generator.
pub mod config_diff;
/// Unit tests for configuration diffing.
#[cfg(test)]
mod config_diff_tests;
/// Unit tests for configuration replication.
#[cfg(test)]
pub mod config_tests;
/// Unit tests for the summarize_conflicts tool.
#[cfg(test)]
mod conflict_summary_tests;
/// Sync conflict management tools.
pub mod conflicts;
/// Connectivity watchdog tools.
pub mod connectivity_watchdog;
/// File diagnostics tools.
pub mod file_diagnostics;
/// Unit tests for file diagnostics tools.
#[cfg(test)]
mod file_diagnostics_tests;
/// Unit tests for connectivity watchdog tools.
#[cfg(test)]
mod connectivity_watchdog_tests;
/// Global dashboard tool.
pub mod dashboard;
/// Unit tests for the dashboard tool.
#[cfg(test)]
mod dashboard_tests;
/// Device management tools.
pub mod devices;
/// Unit tests for the devices tool.
#[cfg(test)]
mod devices_tests;
/// Advanced diffing tools for conflict resolution.
pub mod diff;
/// Unit tests for advanced diffing tools.
#[cfg(test)]
mod diff_tests;
/// Folder management tools.
pub mod folders;
/// Unit tests for the folders tool.
#[cfg(test)]
mod folders_tests;
/// Git-Sync tools for version control.
pub mod git_sync;
/// Unit tests for Git-Sync tools.
#[cfg(test)]
mod git_sync_tests;
/// Unit tests for the inspect_device tool.
#[cfg(test)]
mod inspect_device_tests;
/// Unit tests for the inspect_folder tool.
#[cfg(test)]
mod inspect_folder_tests;
/// Unit tests for the get_instance_overview tool.
#[cfg(test)]
mod instance_overview_tests;
/// Unit tests for performance profiles.
#[cfg(test)]
mod profile_tests;
/// Self-healing monitor tools.
pub mod self_healing;
/// Unit tests for self-healing tools.
#[cfg(test)]
mod self_healing_tests;
/// System status and maintenance tools.
pub mod system;
/// Unit tests for the system tools.
#[cfg(test)]
mod system_tests;

use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;

/// A type alias for tool handler functions.
pub type ToolHandler = dyn Fn(
        &SyncThingClient,
        &AppConfig,
        Option<Value>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send>>
    + Send
    + Sync;

/// Represents an MCP tool.
#[derive(Clone)]
pub struct Tool {
    /// The name of the tool.
    pub name: String,
    /// A description of what the tool does.
    pub description: String,
    /// The JSON schema for the tool's input.
    pub input_schema: Value,
    /// The handler function for the tool.
    pub handler: Arc<ToolHandler>,
}

/// A registry of available MCP tools.
#[derive(Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    enabled_tools: HashSet<String>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Creates a new, empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            enabled_tools: HashSet::new(),
        }
    }

    /// Registers a new tool in the registry.
    pub fn register<F, Fut>(
        &mut self,
        name: &str,
        description: &str,
        input_schema: Value,
        handler: F,
    ) where
        F: Fn(SyncThingClient, AppConfig, Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value>> + Send + 'static,
    {
        let tool = Tool {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
            handler: Arc::new(move |client, config, params| {
                let client = client.clone();
                let config = config.clone();
                let params = params.clone().unwrap_or_default();
                Box::pin(handler(client, config, params))
            }),
        };
        self.tools.insert(name.to_string(), tool.clone());
        self.enabled_tools.insert(name.to_string());
    }

    /// Lists all enabled tools in the registry, formatted for MCP.
    pub fn list_tools(&self) -> Vec<Value> {
        let mut result = Vec::new();

        for tool_name in &self.enabled_tools {
            if let Some(tool) = self.tools.get(tool_name) {
                let mut input_schema = tool.input_schema.clone();
                if let Some(properties) = input_schema
                    .as_object_mut()
                    .and_then(|obj| obj.get_mut("properties"))
                    .and_then(|p| p.as_object_mut())
                {
                    properties.insert(
                        "instance".to_string(),
                        serde_json::json!({
                            "type": "string",
                            "description": "The name or index of the SyncThing instance to target."
                        }),
                    );
                }

                result.push(serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": input_schema
                }));
            }
        }

        result.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        result
    }

    /// Returns a tool by its name.
    pub fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.get(name).cloned()
    }
}

/// Creates a tool registry and registers all available tools.
pub fn create_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    registry.register(
        "get_instance_overview",
        "Provides a top-level health and status report for a SyncThing instance, consolidating system status, connections, and version information.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["text", "json"],
                    "description": "Output format (default: text).",
                    "default": "text"
                },
                "shorten": {
                    "type": "boolean",
                    "description": "If true, use short aliases for fields in JSON output.",
                    "default": true
                },
                "fields": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of fields to include in JSON output."
                }
            }
        }),
        system::get_instance_overview,
    );

    registry.register(
        "list_instances",
        "List all configured SyncThing instances and their current health status.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        system::list_instances,
    );

    registry.register(
        "get_instance_health",
        "Get detailed health information for a specific SyncThing instance, including connectivity, version, uptime, and resource usage.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        system::get_instance_health,
    );

    registry.register(
        "analyze_error",
        "Analyze a technical error message and provide a diagnostic summary with actionable advice.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "error_message": {
                    "type": "string",
                    "description": "The technical error message to analyze."
                },
                "tool_name": {
                    "type": "string",
                    "description": "The name of the tool that failed (optional)."
                }
            },
            "required": ["error_message"]
        }),
        system::analyze_error,
    );

    registry.register(
        "get_system_status",
        "Get comprehensive system status information, including version, uptime, memory usage, and the local device ID.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        system::get_system_stats,
    );

    registry.register(
        "get_system_connections",
        "Get the current connection status and data transfer statistics for all connected devices.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        system::get_system_connections,
    );

    registry.register(
        "get_system_log",
        "Get recent log entries from the SyncThing service for troubleshooting.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        system::get_system_log,
    );

    registry.register(
        "get_sync_status",
        "Get detailed synchronization status, including state, completion percentage, and download/upload rates for a specific folder or device.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "enum": ["folder", "device"],
                    "description": "The target type to query status for."
                },
                "id": {
                    "type": "string",
                    "description": "The unique Folder ID or Device ID to query."
                }
            },
            "required": ["target", "id"]
        }),
        system::get_sync_status,
    );

    registry.register(
        "manage_folders",
        "Manage SyncThing folders: list configured folders, get a specific folder, view pending folder requests, reject pending requests, or revert local changes in Receive Only folders.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "get", "pending", "reject_pending", "revert"],
                    "description": "The folder management action to perform. 'revert' undoes local changes in Receive Only folders."
                },
                "folder_id": {
                    "type": "string",
                    "description": "The unique Folder ID (required for 'get', 'reject_pending' and 'revert')."
                }
            },
            "required": ["action"]
        }),
        folders::manage_folders,
    );

    registry.register(
        "inspect_folder",
        "Provides a comprehensive status overview for a specific folder, consolidating sync status, conflicts, and statistics.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "folder_id": {
                    "type": "string",
                    "description": "The unique Folder ID to inspect."
                },
                "format": {
                    "type": "string",
                    "enum": ["text", "json"],
                    "description": "Output format (default: text).",
                    "default": "text"
                },
                "shorten": {
                    "type": "boolean",
                    "description": "If true, use short aliases for fields in JSON output.",
                    "default": true
                },
                "fields": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of fields to include in JSON output."
                }
            },
            "required": ["folder_id"]
        }),
        folders::inspect_folder,
    );

    registry.register(
        "batch_manage_folders",
        "Performs bulk actions (rescan, revert, pause, resume) on multiple folders simultaneously.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "folder_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of folder IDs to apply the action to."
                },
                "action": {
                    "type": "string",
                    "enum": ["rescan", "revert", "pause", "resume"],
                    "description": "The action to perform on each folder."
                },
                "format": {
                    "type": "string",
                    "enum": ["text", "json"],
                    "description": "Output format (default: text).",
                    "default": "text"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of items to return in arrays.",
                    "default": 20
                }
            },
            "required": ["folder_ids", "action"]
        }),
        folders::batch_manage_folders,
    );

    registry.register(
        "configure_sharing",
        "Configure folder sharing between devices (share or unshare).",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["share", "unshare"],
                    "description": "Whether to share or unshare the folder."
                },
                "folder_id": {
                    "type": "string",
                    "description": "The ID of the folder to configure."
                },
                "device_id": {
                    "type": "string",
                    "description": "The ID of the device to share/unshare with."
                }
            },
            "required": ["action", "folder_id", "device_id"]
        }),
        folders::configure_sharing,
    );

    registry.register(
        "manage_ignores",
        "Manage folder ignore patterns (.stignore). Supports getting current patterns, setting a new list, or appending to the existing list.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["get", "set", "append"],
                    "description": "The ignore management action to perform."
                },
                "folder_id": {
                    "type": "string",
                    "description": "The ID of the folder whose ignores should be managed."
                },
                "patterns": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "The list of ignore patterns (required for 'set' and 'append')."
                }
            },
            "required": ["folder_id", "action"]
        }),
        folders::manage_ignores,
    );

    registry.register(
        "inspect_device",
        "Provides a comprehensive status overview for a specific device, consolidating completion status and statistics.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "device_id": {
                    "type": "string",
                    "description": "The unique Device ID to inspect."
                },
                "format": {
                    "type": "string",
                    "enum": ["text", "json"],
                    "description": "Output format (default: text).",
                    "default": "text"
                },
                "shorten": {
                    "type": "boolean",
                    "description": "If true, use short aliases for fields in JSON output.",
                    "default": true
                }
            },
            "required": ["device_id"]
        }),
        devices::inspect_device,
    );

    registry.register(
        "manage_devices",
        "Manage SyncThing devices: list, add, remove, pause, resume, approve pending devices, or validate device IDs.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "add", "remove", "pause", "resume", "discover", "approve", "validate"],
                    "description": "The device management action to perform. 'discover' lists pending device requests."
                },
                "device_id": {
                    "type": "string",
                    "description": "The unique Device ID (required for all actions except 'list' and 'discover')."
                },
                "name": {
                    "type": "string",
                    "description": "The friendly name for the device (optional, used for 'add' or 'approve')."
                }
            },
            "required": ["action"]
        }),
        devices::manage_devices,
    );

    registry.register(
        "get_device_statistics",
        "Get detailed connection statistics for all devices, including last seen time and last connection duration.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        devices::get_device_stats,
    );

    registry.register(
        "get_folder_statistics",
        "Get detailed statistics for all folders, including last scan time and information about the last synced file.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        folders::get_folder_stats,
    );

    registry.register(
        "get_file_info",
        "Get detailed metadata and availability information for a specific file in a folder.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "folder_id": {
                    "type": "string",
                    "description": "The ID of the folder containing the file."
                },
                "file_path": {
                    "type": "string",
                    "description": "The relative path to the file within the folder."
                }
            },
            "required": ["folder_id", "file_path"]
        }),
        file_diagnostics::get_file_info,
    );

    registry.register(
        "get_folder_needs",
        "Get the list of files that are needed to bring a folder up to date.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "folder_id": {
                    "type": "string",
                    "description": "The ID of the folder to query."
                },
                "page": {
                    "type": "integer",
                    "description": "The page number to retrieve (optional)."
                },
                "per_page": {
                    "type": "integer",
                    "description": "The number of items per page (optional)."
                }
            },
            "required": ["folder_id"]
        }),
        file_diagnostics::get_folder_needs,
    );

    registry.register(
        "maintain_system",
        "Perform system maintenance: force a rescan of folders, restart the SyncThing service, or shut down the service.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["force_rescan", "restart", "shutdown", "clear_errors"],
                    "description": "The maintenance action to perform."
                },
                "folder_id": {
                    "type": "string",
                    "description": "Optional specific Folder ID for 'force_rescan'. If omitted, all folders are rescanned."
                }
            },
            "required": ["action"]
        }),
        system::maintain_system,
    );

    registry.register(
        "replicate_config",
        "Replicate configuration (folders and devices) from one SyncThing instance to another. Optionally perform a dry run or select specific folders/devices.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "Name or index of the source SyncThing instance (defaults to the first instance)."
                },
                "destination": {
                    "type": "string",
                    "description": "Name or index of the destination SyncThing instance."
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "If true, preview changes without applying them.",
                    "default": false
                },
                "folders": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional list of folder IDs to replicate. If omitted, all folders are replicated."
                },
                "devices": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional list of device IDs to replicate. If omitted, all devices are replicated."
                }
            },
            "required": ["destination"]
        }),
        config::replicate_config,
    );

    registry.register(
        "diff_instance_configs",
        "Returns a detailed difference report between two SyncThing instance configurations.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "Name or index of the source SyncThing instance (defaults to the first instance)."
                },
                "destination": {
                    "type": "string",
                    "description": "Name or index of the destination SyncThing instance."
                }
            },
            "required": ["destination"]
        }),
        config::diff_instance_configs,
    );

    registry.register(
        "merge_instance_configs",
        "Merges configuration from one SyncThing instance into another. This appends/updates folders and devices instead of replacing the entire configuration.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "Name or index of the source SyncThing instance (defaults to the first instance)."
                },
                "destination": {
                    "type": "string",
                    "description": "Name or index of the destination SyncThing instance."
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "If true, preview changes without applying them.",
                    "default": false
                }
            },
            "required": ["destination"]
        }),
        config::merge_instance_configs,
    );

    registry.register(
        "browse_folder",
        "Browse the contents of a synced folder, listing files and subdirectories with optional prefix and recursion depth control.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "folder_id": {
                    "type": "string",
                    "description": "The ID of the folder to browse."
                },
                "prefix": {
                    "type": "string",
                    "description": "Optional subdirectory path to start browsing from."
                },
                "levels": {
                    "type": "integer",
                    "description": "How many levels deep to traverse (0 for immediate contents only)."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of items to return in arrays.",
                    "default": 100
                },
                "shorten": {
                    "type": "boolean",
                    "description": "If true, use short aliases for fields in JSON output.",
                    "default": true
                }
            },
            "required": ["folder_id"]
        }),
        browser::browse_folder,
    );

    registry.register(
        "list_conflicts",
        "List SyncThing conflict files in a specific folder.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "folder_id": {
                    "type": "string",
                    "description": "The ID of the folder to scan for conflicts."
                }
            },
            "required": ["folder_id"]
        }),
        conflicts::list_conflicts,
    );

    registry.register(
        "summarize_conflicts",
        "Provides an actionable summary of conflicts across all folders, grouped by folder with counts and sizes.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["text", "json"],
                    "description": "Output format (default: text).",
                    "default": "text"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of items to return in arrays.",
                    "default": 20
                },
                "shorten": {
                    "type": "boolean",
                    "description": "If true, use short aliases for fields in JSON output.",
                    "default": true
                }
            }
        }),
        conflicts::summarize_conflicts,
    );

    registry.register(
        "resolve_conflict",
        "Resolves a SyncThing conflict by keeping either the original or the conflict version.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "conflict_path": {
                    "type": "string",
                    "description": "The full path to the conflict file."
                },
                "action": {
                    "type": "string",
                    "enum": ["keep_original", "keep_conflict"],
                    "description": "The resolution action: 'keep_original' (deletes conflict) or 'keep_conflict' (replaces original with conflict)."
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "If true, preview the action without performing any file operations.",
                    "default": false
                },
                "backup": {
                    "type": "boolean",
                    "description": "If true, move the overwritten or deleted file to the system trash instead of permanent deletion.",
                    "default": true
                },
                "preview": {
                    "type": "boolean",
                    "description": "If true, return a preview of the resolution without performing any file operations.",
                    "default": false
                }
            },
            "required": ["conflict_path", "action"]
        }),
        conflicts::resolve_conflict,
    );

    registry.register(
        "delete_conflict",
        "Permanently delete a SyncThing conflict file.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "conflict_path": {
                    "type": "string",
                    "description": "The full path to the conflict file."
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "If true, preview the deletion without performing any file operations.",
                    "default": false
                },
                "backup": {
                    "type": "boolean",
                    "description": "If true, move the deleted file to the system trash instead of permanent deletion.",
                    "default": true
                }
            },
            "required": ["conflict_path"]
        }),
        conflicts::delete_conflict,
    );

    registry.register(
        "diff_conflicts",
        "Compare the original and conflict versions of a file.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "conflict_path": {
                    "type": "string",
                    "description": "The full path to the conflict file."
                },
                "format": {
                    "type": "string",
                    "enum": ["auto", "text", "json", "yaml"],
                    "description": "The format of the files (default: auto).",
                    "default": "auto"
                }
            },
            "required": ["conflict_path"]
        }),
        diff::diff_conflicts,
    );

    registry.register(
        "preview_conflict_resolution",
        "Show what the file will look like after a proposed resolution.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "conflict_path": {
                    "type": "string",
                    "description": "The full path to the conflict file."
                },
                "action": {
                    "type": "string",
                    "enum": ["keep_original", "keep_conflict"],
                    "description": "The resolution action to preview."
                }
            },
            "required": ["conflict_path", "action"]
        }),
        diff::preview_conflict_resolution,
    );

    registry.register(
        "get_global_dashboard",
        "Get a high-level overview of all configured SyncThing instances, including aggregated transfer rates and network health.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        dashboard::get_global_dashboard,
    );

    registry.register(
        "monitor_self_healing",
        "Monitor tool that checks for stuck folders and disconnected devices, and triggers self-healing actions.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "dry_run": {
                    "type": "boolean",
                    "description": "If true, only report what actions would be taken without actually performing them.",
                    "default": false
                }
            }
        }),
        self_healing::monitor_self_healing,
    );

    registry.register(
        "set_bandwidth_limits",
        "Set the bandwidth limits (upload/download) across one or all SyncThing instances.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "max_recv_kbps": {
                    "type": "integer",
                    "description": "Maximum receive rate in Kbps. Set to 0 for unlimited."
                },
                "max_send_kbps": {
                    "type": "integer",
                    "description": "Maximum send rate in Kbps. Set to 0 for unlimited."
                }
            }
        }),
        bandwidth::set_bandwidth_limits,
    );

    registry.register(
        "set_performance_profile",
        "Set the active performance profile (e.g., 'working_hours', 'overnight', 'full_speed').",
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name of the performance profile to activate."
                }
            },
            "required": ["name"]
        }),
        bandwidth::set_performance_profile,
    );

    registry.register(
        "get_bandwidth_status",
        "Get current bandwidth limits and active profiles for all SyncThing instances.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        bandwidth::get_bandwidth_status,
    );

    registry
}
