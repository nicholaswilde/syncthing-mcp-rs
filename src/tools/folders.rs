use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde_json::{json, Value};

pub async fn manage_folders(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let action = args["action"].as_str().unwrap_or("list");

    match action {
        "list" => {
            let folders = client.list_folders().await?;
            let mut text = String::from("SyncThing Folders:\n");
            for folder in folders {
                text.push_str(&format!(
                    "- {} ({}): {} (paused: {})\n",
                    folder.label, folder.id, folder.path, folder.paused
                ));
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        _ => Err(crate::error::Error::Internal(format!("Unsupported action: {}", action))),
    }
}

pub async fn configure_sharing(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let action = args["action"].as_str().ok_or_else(|| crate::error::Error::Internal("action is required".to_string()))?;
    let folder_id = args["folder_id"].as_str().ok_or_else(|| crate::error::Error::Internal("folder_id is required".to_string()))?;
    let device_id = args["device_id"].as_str().ok_or_else(|| crate::error::Error::Internal("device_id is required".to_string()))?;

    let mut folder = client.get_folder(folder_id).await?;

    match action {
        "share" => {
            if !folder.devices.iter().any(|d| d.device_id == device_id) {
                folder.devices.push(crate::api::models::FolderDeviceConfiguration {
                    device_id: device_id.to_string(),
                });
                client.patch_folder(folder_id, json!({"devices": folder.devices})).await?;
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Folder {} shared with device {} successfully", folder_id, device_id)
                }]
            }))
        }
        "unshare" => {
            let original_len = folder.devices.len();
            folder.devices.retain(|d| d.device_id != device_id);
            if folder.devices.len() != original_len {
                client.patch_folder(folder_id, json!({"devices": folder.devices})).await?;
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Folder {} unshared from device {} successfully", folder_id, device_id)
                }]
            }))
        }
        _ => Err(crate::error::Error::Internal(format!("Unsupported action: {}", action))),
    }
}
