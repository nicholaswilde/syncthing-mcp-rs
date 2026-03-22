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
