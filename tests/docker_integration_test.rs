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
