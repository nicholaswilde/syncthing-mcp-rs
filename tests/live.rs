use anyhow::Result;
use serde_json::json;
use std::sync::{Arc, Mutex};
use syncthing_mcp_rs::api::SyncThingClient;
use syncthing_mcp_rs::config::AppConfig;
use syncthing_mcp_rs::tools::ToolRegistry;

pub struct LiveTestContext {
    pub config: AppConfig,
    pub client: SyncThingClient,
    pub registry: Arc<Mutex<ToolRegistry>>,
}

impl LiveTestContext {
    pub async fn new() -> Result<Self> {
        let api_key = std::env::var("SYNCTHING_API_KEY")?;
        let host = std::env::var("SYNCTHING_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("SYNCTHING_PORT")
            .unwrap_or_else(|_| "8384".to_string())
            .parse::<u16>()?;
        let _url = if host.starts_with("http") {
            host.clone()
        } else {
            format!("http://{}:{}", host, port)
        };

        let mut app_config = AppConfig {
            host: host.clone(),
            port,
            api_key: Some(api_key.clone()),
            ..Default::default()
        };
        app_config
            .validate()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let client = SyncThingClient::new(app_config.instances[0].clone());
        let registry = Arc::new(Mutex::new(syncthing_mcp_rs::tools::create_registry()));

        Ok(Self {
            config: app_config,
            client,
            registry,
        })
    }

    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let handler = {
            let reg = self.registry.lock().unwrap();
            reg.get_tool(name).map(|t| t.handler.clone())
        };

        if let Some(handler) = handler {
            handler(&self.client, &self.config, Some(args))
                .await
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            Err(anyhow::anyhow!("Tool not found: {}", name))
        }
    }
}

#[tokio::test]
async fn test_live_system_status() -> Result<()> {
    if std::env::var("RUN_LIVE_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = LiveTestContext::new().await?;
    let status = ctx.client.get_system_status().await?;
    println!("Live System Status: {:?}", status);
    assert!(!status.my_id.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_live_list_folders() -> Result<()> {
    if std::env::var("RUN_LIVE_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = LiveTestContext::new().await?;
    let result = ctx
        .call_tool("manage_folders", json!({"action": "list"}))
        .await?;
    println!(
        "Live Folders:\n{}",
        result["content"][0]["text"].as_str().unwrap()
    );
    Ok(())
}

#[tokio::test]
async fn test_live_get_system_stats() -> Result<()> {
    if std::env::var("RUN_LIVE_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = LiveTestContext::new().await?;
    let result = ctx.call_tool("get_system_status", json!({})).await?;
    println!(
        "Live System Stats:\n{}",
        result["content"][0]["text"].as_str().unwrap()
    );
    Ok(())
}

#[tokio::test]
async fn test_live_get_sync_status() -> Result<()> {
    if std::env::var("RUN_LIVE_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = LiveTestContext::new().await?;

    // 1. Test system status to get local device ID
    let status = ctx.client.get_system_status().await?;
    let my_id = status.my_id;

    // 2. Test device sync status
    let result = ctx
        .call_tool(
            "get_sync_status",
            json!({
                "target": "device",
                "id": my_id
            }),
        )
        .await?;
    println!(
        "Live Device Sync Status:\n{}",
        result["content"][0]["text"].as_str().unwrap()
    );

    // 3. Test folder sync status (using first folder from list)
    let folders = ctx.client.list_folders().await?;
    if let Some(folder) = folders.first() {
        let result = ctx
            .call_tool(
                "get_sync_status",
                json!({
                    "target": "folder",
                    "id": folder.id
                }),
            )
            .await?;
        println!(
            "Live Folder Sync Status ({}):\n{}",
            folder.id,
            result["content"][0]["text"].as_str().unwrap()
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_live_browse_folder() -> Result<()> {
    if std::env::var("RUN_LIVE_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = LiveTestContext::new().await?;
    let folders = ctx.client.list_folders().await?;
    if let Some(folder) = folders.first() {
        let result = ctx
            .call_tool(
                "browse_folder",
                json!({
                    "folder_id": folder.id,
                    "levels": 1
                }),
            )
            .await?;
        println!(
            "Live Browse Folder ({}):\n{}",
            folder.id,
            serde_json::to_string_pretty(&result).unwrap()
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_live_manage_devices() -> Result<()> {
    if std::env::var("RUN_LIVE_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = LiveTestContext::new().await?;
    let result = ctx
        .call_tool("manage_devices", json!({"action": "list"}))
        .await?;
    println!(
        "Live Devices:\n{}",
        result["content"][0]["text"].as_str().unwrap()
    );
    Ok(())
}

#[tokio::test]
async fn test_live_manage_ignores() -> Result<()> {
    if std::env::var("RUN_LIVE_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = LiveTestContext::new().await?;
    let folders = ctx.client.list_folders().await?;
    if let Some(folder) = folders.first() {
        let result = ctx
            .call_tool(
                "manage_ignores",
                json!({
                    "action": "get",
                    "folder_id": folder.id
                }),
            )
            .await?;
        println!(
            "Live Ignores ({}):\n{}",
            folder.id,
            result["content"][0]["text"].as_str().unwrap()
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_live_get_events() -> Result<()> {
    if std::env::var("RUN_LIVE_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = LiveTestContext::new().await?;
    let events = ctx.client.get_events(None, Some(5)).await?;
    println!("Live Recent Events:");
    for event in events {
        println!(
            "- [ID: {}] {}: {}",
            event.id,
            event.event_type,
            event.summary()
        );
    }
    Ok(())
}
