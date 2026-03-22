pub mod browser;
pub mod config;
pub mod devices;
pub mod folders;
pub mod system;

use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;

pub type ToolHandler = dyn Fn(
        &SyncThingClient,
        &AppConfig,
        Option<Value>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send>>
    + Send
    + Sync;

#[derive(Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub handler: Arc<ToolHandler>,
}

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
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            enabled_tools: HashSet::new(),
        }
    }

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

    pub fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.get(name).cloned()
    }
}

pub fn create_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    registry.register(
        "get_system_stats",
        "Get SyncThing system statistics and version.",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        system::get_system_stats,
    );

    registry.register(
        "get_sync_status",
        "Get detailed synchronization status and completion percentage for a folder or device.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "enum": ["folder", "device"],
                    "description": "The target to query status for."
                },
                "id": {
                    "type": "string",
                    "description": "The Folder ID or Device ID."
                }
            },
            "required": ["target", "id"]
        }),
        system::get_sync_status,
    );

    registry.register(
        "manage_folders",
        "List SyncThing folders.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list"],
                    "description": "The action to perform."
                }
            }
        }),
        folders::manage_folders,
    );

    registry.register(
        "configure_sharing",
        "Share or unshare a folder with a device.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["share", "unshare"],
                    "description": "The action to perform."
                },
                "folder_id": {
                    "type": "string",
                    "description": "The Folder ID."
                },
                "device_id": {
                    "type": "string",
                    "description": "The Device ID."
                }
            },
            "required": ["action", "folder_id", "device_id"]
        }),
        folders::configure_sharing,
    );

    registry.register(
        "manage_ignores",
        "Manage SyncThing ignore patterns (.stignore).",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["get", "set", "append"],
                    "description": "The action to perform."
                },
                "folder_id": {
                    "type": "string",
                    "description": "The Folder ID."
                },
                "patterns": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "The ignore patterns (required for 'set' and 'append')."
                }
            },
            "required": ["folder_id"]
        }),
        folders::manage_ignores,
    );

    registry.register(
        "manage_devices",
        "Manage SyncThing devices.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "add", "remove", "pause", "resume", "discover", "approve"],
                    "description": "The action to perform."
                },
                "device_id": {
                    "type": "string",
                    "description": "The Device ID (required for 'remove', 'pause', 'resume', 'approve')."
                },
                "name": {
                    "type": "string",
                    "description": "The device name (optional)."
                }
            },
            "required": ["action"]
        }),
        devices::manage_devices,
    );

    registry.register(
        "maintain_system",
        "Perform maintenance tasks on the SyncThing instance.",
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
                    "description": "Optional Folder ID for 'force_rescan'. If omitted, all folders are rescanned."
                }
            },
            "required": ["action"]
        }),
        system::maintain_system,
    );

    registry.register(
        "replicate_config",
        "Sync configuration (folders and devices) from one SyncThing instance to another.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "Source instance name or index (optional, defaults to current/first)."
                },
                "destination": {
                    "type": "string",
                    "description": "Destination instance name or index."
                }
            },
            "required": ["destination"]
        }),
        config::replicate_config,
    );

    registry.register(
        "browse_folder",
        "List files and subdirectories within a synced folder.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "folder_id": {
                    "type": "string",
                    "description": "The Folder ID."
                },
                "prefix": {
                    "type": "string",
                    "description": "Optional path prefix within the folder."
                },
                "levels": {
                    "type": "integer",
                    "description": "How deep to traverse (0 for current level only)."
                }
            },
            "required": ["folder_id"]
        }),
        browser::browse_folder,
    );

    registry
}
