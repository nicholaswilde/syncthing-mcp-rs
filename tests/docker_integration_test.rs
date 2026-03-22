mod common;

use anyhow::Result;
use common::{SyncThingContainer, TestContext};
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
    let result = ctx
        .call_tool("manage_folders", json!({"action": "list"}))
        .await?;

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
    let result = ctx
        .call_tool("manage_devices", json!({"action": "list"}))
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("SyncThing Devices:"));

    // 2. Add a device
    let dummy_id = "PIRQAMB-72MHUAV-UZDMOA4-GXFI6LX-SVYUDGG-YIXLXHE-FW4CCMO-6KVZAA3";
    let result = ctx
        .call_tool(
            "manage_devices",
            json!({
                "action": "add",
                "device_id": dummy_id,
                "name": "Dummy Device"
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("added successfully"));

    // 3. Pause the device
    let result = ctx
        .call_tool(
            "manage_devices",
            json!({
                "action": "pause",
                "device_id": dummy_id
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("paused successfully"));

    // 4. Resume the device
    let result = ctx
        .call_tool(
            "manage_devices",
            json!({
                "action": "resume",
                "device_id": dummy_id
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("resumed successfully"));

    // 5. Remove the device
    let result = ctx
        .call_tool(
            "manage_devices",
            json!({
                "action": "remove",
                "device_id": dummy_id
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("removed successfully"));

    Ok(())
}

#[tokio::test]
async fn test_configure_sharing_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = SyncThingContainer::new().await?;
    let client = container.client();

    let folder_id = "test-sharing-folder";
    let dummy_id = "PIRQAMB-72MHUAV-UZDMOA4-GXFI6LX-SVYUDGG-YIXLXHE-FW4CCMO-6KVZAA3";

    // 1. Create a folder
    client
        .add_folder(folder_id, "Test Sharing Folder", "/tmp/test-sharing")
        .await?;

    let ctx = TestContext::from_container(container);

    // 2. Share folder with device
    let result = ctx
        .call_tool(
            "configure_sharing",
            json!({
                "action": "share",
                "folder_id": folder_id,
                "device_id": dummy_id
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("shared with device"));

    // 3. Unshare folder from device
    let result = ctx
        .call_tool(
            "configure_sharing",
            json!({
                "action": "unshare",
                "folder_id": folder_id,
                "device_id": dummy_id
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("unshared from device"));

    Ok(())
}

#[tokio::test]
async fn test_manage_ignores_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = SyncThingContainer::new().await?;
    let client = container.client();

    let folder_id = "test-ignores-folder";

    // 1. Create a folder
    client
        .add_folder(folder_id, "Test Ignores Folder", "/tmp/test-ignores")
        .await?;

    let ctx = TestContext::from_container(container);

    // 2. Get ignores (should be empty)
    let result = ctx
        .call_tool(
            "manage_ignores",
            json!({
                "action": "get",
                "folder_id": folder_id
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("(No ignore patterns set)"));

    // 3. Set ignores
    let result = ctx
        .call_tool(
            "manage_ignores",
            json!({
                "action": "set",
                "folder_id": folder_id,
                "patterns": ["node_modules", "*.tmp"]
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Successfully set 2 ignore patterns"));

    // 4. Append ignores
    let result = ctx
        .call_tool(
            "manage_ignores",
            json!({
                "action": "append",
                "folder_id": folder_id,
                "patterns": ["target", "*.tmp"] // *.tmp is duplicate, target is new
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Successfully appended 1 new ignore patterns"));
    assert!(text.contains("Total: 3"));

    // 5. Verify final list
    let result = ctx
        .call_tool(
            "manage_ignores",
            json!({
                "action": "get",
                "folder_id": folder_id
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("- node_modules"));
    assert!(text.contains("- *.tmp"));
    assert!(text.contains("- target"));

    Ok(())
}

#[tokio::test]
async fn test_get_sync_status_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = SyncThingContainer::new().await?;
    let client = container.client();

    let folder_id = "test-status-folder";

    // 1. Create a folder so we have something to check
    client
        .add_folder(folder_id, "Test Status Folder", "/tmp/test-status")
        .await?;

    let ctx = TestContext::from_container(container);

    // 2. Test folder status
    let result = ctx
        .call_tool(
            "get_sync_status",
            json!({
                "target": "folder",
                "id": folder_id
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains(&format!("Folder: {}", folder_id)));
    assert!(text.contains("Completion:"));
    assert!(text.contains("State:"));

    // 3. Test device status (using the local device ID since we know it exists)
    let status = client.get_system_status().await?;
    let my_id = status.my_id;

    let result = ctx
        .call_tool(
            "get_sync_status",
            json!({
                "target": "device",
                "id": my_id
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains(&format!("Device: {}", my_id)));
    assert!(text.contains("Completion:"));

    Ok(())
}

#[tokio::test]
async fn test_maintain_system_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").is_err() {
        println!("Skipping Docker test. Set RUN_DOCKER_TESTS=1 to run.");
        return Ok(());
    }

    let ctx = TestContext::new().await?;

    // 1. Force rescan
    let result = ctx
        .call_tool("maintain_system", json!({"action": "force_rescan"}))
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Successfully triggered rescan"));

    // 2. Clear errors
    let result = ctx
        .call_tool("maintain_system", json!({"action": "clear_errors"}))
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Successfully cleared SyncThing errors"));

    // 3. Restart
    let result = ctx
        .call_tool("maintain_system", json!({"action": "restart"}))
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Successfully triggered SyncThing restart"));

    Ok(())
}
