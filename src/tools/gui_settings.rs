use crate::api::client::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::{Value, json};

/// Tool handler for retrieving GUI settings.
pub async fn get_gui_settings(
    client: SyncThingClient,
    _config: AppConfig,
    _args: Value,
) -> Result<Value> {
    let gui_config = client.get_gui_config().await?;

    // Mask sensitive fields before returning via MCP
    let mut masked_config = gui_config;
    if masked_config.user.is_some() {
        masked_config.user = Some("********".to_string());
    }
    if masked_config.password.is_some() {
        masked_config.password = Some("********".to_string());
    }
    if masked_config.api_key.is_some() {
        masked_config.api_key = Some("********".to_string());
    }

    Ok(json!(masked_config))
}

/// Tool handler for updating GUI settings.
pub async fn update_gui_settings(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let mut gui_config = client.get_gui_config().await?;

    if let Some(enabled) = args.get("enabled").and_then(|v| v.as_bool()) {
        gui_config.enabled = enabled;
    }
    if let Some(address) = args.get("address").and_then(|v| v.as_str()) {
        gui_config.address = address.to_string();
    }
    if let Some(user) = args.get("user").and_then(|v| v.as_str()) {
        if user.is_empty() {
            gui_config.user = None;
        } else {
            gui_config.user = Some(user.to_string());
        }
    }
    if let Some(password) = args.get("password").and_then(|v| v.as_str()) {
        if password.is_empty() {
            gui_config.password = None;
        } else {
            gui_config.password = Some(password.to_string());
        }
    }
    if let Some(use_tls) = args.get("useTLS").and_then(|v| v.as_bool()) {
        gui_config.use_tls = use_tls;
    }
    if let Some(theme) = args.get("theme").and_then(|v| v.as_str()) {
        gui_config.theme = theme.to_string();
    }

    client.set_gui_config(&gui_config).await?;

    Ok(json!({ "message": "GUI settings updated successfully" }))
}
