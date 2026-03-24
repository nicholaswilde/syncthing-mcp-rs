/// Browser tool for exploring synced folders.
pub mod browser;
/// Sync conflict management tools.
pub mod conflicts;
/// Configuration replication tool.
pub mod config;
/// Device management tools.
pub mod devices;
/// Folder management tools.
pub mod folders;
/// System status and maintenance tools.
pub mod system;

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
        "get_system_stats",
        "Get SyncThing system statistics, including version, uptime, memory usage, and the unique device ID.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        system::get_system_stats,
    );

    registry.register(
        "get_sync_status",
        "Get detailed synchronization status, state, and completion percentage for a specific folder or device.",
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
        "List all configured SyncThing folders, showing their IDs, labels, paths, and paused status.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list"],
                    "description": "The folder management action to perform."
                }
            }
        }),
        folders::manage_folders,
    );

    registry.register(
        "configure_sharing",
        "Share or unshare a specific folder with a remote device.",
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
        "Manage SyncThing ignore patterns (.stignore). Supports getting current patterns, setting a new list, or appending to the existing list.",
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
            "required": ["folder_id"]
        }),
        folders::manage_ignores,
    );

    registry.register(
        "manage_devices",
        "Manage SyncThing devices, including listing, adding, removing, pausing, resuming, and approving pending devices.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "add", "remove", "pause", "resume", "discover", "approve"],
                    "description": "The device management action to perform."
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
        "maintain_system",
        "Perform system maintenance: force a rescan of folders, restart the SyncThing service, or clear internal errors.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["force_rescan", "restart", "clear_errors"],
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
        "Replicate folder and device configurations from one SyncThing instance to another for easy synchronization setup.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "The name or index of the source SyncThing instance (defaults to the first instance)."
                },
                "destination": {
                    "type": "string",
                    "description": "The name or index of the destination SyncThing instance."
                }
            },
            "required": ["destination"]
        }),
        config::replicate_config,
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
        "resolve_conflict",
        "Resolve a SyncThing conflict file by keeping either the original or the conflict version.",
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
                }
            },
            "required": ["conflict_path", "action"]
        }),
        conflicts::resolve_conflict,
    );

    registry
}
