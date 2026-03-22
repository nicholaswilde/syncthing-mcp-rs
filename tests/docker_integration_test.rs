mod common;

use common::{SyncThingContainer, TestContext};
use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_container_starts() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = SyncThingContainer::new().await?;
    let status = container.client().get_system_status().await?;
    
    assert!(!status.my_id.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_get_system_stats_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;
    let result = ctx.call_tool("get_system_stats", json!({})).await?;
    
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("SyncThing Version"));
    assert!(text.contains("My ID"));
    
    Ok(())
}

#[tokio::test]
async fn test_manage_folders_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;
    let result = ctx.call_tool("manage_folders", json!({"action": "list"})).await?;
    
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("SyncThing Folders:"));
    
    Ok(())
}

#[tokio::test]
async fn test_auth_failure() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = SyncThingContainer::new().await?;
    let mut config = container.instance_config();
    config.api_key = Some("invalid-api-key".to_string());
    
    let client = syncthing_mcp_rs::api::SyncThingClient::new(config);
    let result = client.get_system_status().await;
    
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_manage_devices_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;
    
    // 1. List devices (should have at least the local device)
    let result = ctx.call_tool("manage_devices", json!({"action": "list"})).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("SyncThing Devices:"));

    // 2. Add a device
    let dummy_id = "PIRQAMB-72MHUAV-UZDMOA4-GXFI6LX-SVYUDGG-YIXLXHE-FW4CCMO-6KVZAA3";
    let result = ctx.call_tool("manage_devices", json!({
        "action": "add",
        "device_id": dummy_id,
        "name": "Dummy Device"
    })).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("added successfully"));

    // 3. Pause the device
    let result = ctx.call_tool("manage_devices", json!({
        "action": "pause",
        "device_id": dummy_id
    })).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("paused successfully"));

    // 4. Resume the device
    let result = ctx.call_tool("manage_devices", json!({
        "action": "resume",
        "device_id": dummy_id
    })).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("resumed successfully"));

    // 5. Remove the device
    let result = ctx.call_tool("manage_devices", json!({
        "action": "remove",
        "device_id": dummy_id
    })).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("removed successfully"));

    Ok(())
}
