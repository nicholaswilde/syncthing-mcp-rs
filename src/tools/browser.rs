use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::Value;

pub async fn browse_folder(
    client: SyncThingClient,
    _config: AppConfig,
    params: Value,
) -> Result<Value> {
    let folder = params
        .get("folder_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::error::Error::ValidationError("folder_id is required".to_string()))?;

    let prefix = params.get("prefix").and_then(|v| v.as_str());
    let levels = params.get("levels").and_then(|v| v.as_u64()).map(|v| v as u32);

    let result = client.browse(folder, prefix, levels).await?;
    
    Ok(result)
}
