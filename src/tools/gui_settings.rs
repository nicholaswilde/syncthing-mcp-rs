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
