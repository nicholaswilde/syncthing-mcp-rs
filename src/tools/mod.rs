pub mod folders;
pub mod system;
pub mod devices;

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
        "manage_devices",
        "Manage SyncThing devices.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "add", "remove", "pause", "resume"],
                    "description": "The action to perform."
                },
                "device_id": {
                    "type": "string",
                    "description": "The Device ID."
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

    registry
}
