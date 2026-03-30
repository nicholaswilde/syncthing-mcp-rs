mod common;

use anyhow::Result;
use common::{SyncThingContainer, TestContext};
use serde_json::json;
use std::sync::Arc;

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
async fn test_get_system_status_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;
    let result = ctx.call_tool("get_system_status", json!({})).await?;

    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("SyncThing Version"));
    assert!(text.contains("My ID"));

    Ok(())
}

#[tokio::test]
async fn test_list_instances_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;
    let result = ctx.call_tool("list_instances", json!({})).await?;

    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("SyncThing Instances Status"));
    assert!(text.contains("🟢 Online"));
    assert!(text.contains("Version"));

    Ok(())
}

#[tokio::test]
async fn test_get_instance_health_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;
    let result = ctx.call_tool("get_instance_health", json!({})).await?;

    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("SyncThing Health"));
    assert!(text.contains("🟢 Online"));
    assert!(text.contains("Uptime"));
    assert!(text.contains("Memory Alloc"));

    Ok(())
}

#[tokio::test]
async fn test_get_global_dashboard_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;
    let result = ctx.call_tool("get_global_dashboard", json!({})).await?;

    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Global SyncThing Dashboard"));
    assert!(text.contains("instances online"));
    assert!(text.contains("🟢"));

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
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
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

#[tokio::test]
async fn test_replicate_config_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    use syncthing_mcp_rs::config::InstanceConfig;

    let source_container = SyncThingContainer::new().await?;
    let dest_container = SyncThingContainer::new().await?;

    let source_config = source_container.instance_config();
    let dest_config = dest_container.instance_config();

    let mut app_config = syncthing_mcp_rs::config::AppConfig {
        instances: vec![
            InstanceConfig {
                name: Some("source".to_string()),
                ..source_config
            },
            InstanceConfig {
                name: Some("dest".to_string()),
                ..dest_config
            },
        ],
        ..Default::default()
    };
    app_config.validate().await.unwrap();

    let client = source_container.client();
    let registry = Arc::new(std::sync::Mutex::new(
        syncthing_mcp_rs::tools::create_registry(),
    ));

    // 1. Add a folder to source
    client
        .add_folder("test-replica", "Test Replica", "/tmp/replica")
        .await?;

    // 2. Call replicate_config
    let handler = {
        let reg = registry.lock().unwrap();
        reg.get_tool("replicate_config").unwrap().handler.clone()
    };

    let result = handler(
        &client,
        &app_config,
        Some(json!({
            "source": "source",
            "destination": "dest"
        })),
    )
    .await?;

    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Successfully replicated configuration to dest"));
    assert!(text.contains("Folders: 1 added"));

    // 3. Verify on destination
    let dest_client = dest_container.client();
    let dest_folders = dest_client.list_folders().await?;
    assert!(dest_folders.iter().any(|f| f.id == "test-replica"));

    Ok(())
}

#[tokio::test]
async fn test_diff_instance_configs_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let source_container = SyncThingContainer::new().await?;
    let dest_container = SyncThingContainer::new().await?;

    let source_config = source_container.instance_config();
    let dest_config = dest_container.instance_config();

    let mut app_config = syncthing_mcp_rs::config::AppConfig {
        instances: vec![
            syncthing_mcp_rs::config::InstanceConfig {
                name: Some("source".to_string()),
                ..source_config
            },
            syncthing_mcp_rs::config::InstanceConfig {
                name: Some("dest".to_string()),
                ..dest_config
            },
        ],
        ..Default::default()
    };
    app_config.validate().await.unwrap();

    let source_client = source_container.client();

    // 1. Add a folder to source
    source_client
        .add_folder("test-diff", "Test Diff", "/tmp/diff")
        .await?;

    let ctx = TestContext::from_container(source_container);
    // Overwrite the context's config with our multi-instance config
    let mut ctx = ctx;
    ctx.config = app_config;

    // 2. Call diff_instance_configs
    let result = ctx
        .call_tool(
            "diff_instance_configs",
            json!({
                "source": "source",
                "destination": "dest"
            }),
        )
        .await?;

    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Folders: 1 added, 0 removed, 0 updated."));
    assert!(text.contains("+ Folder: test-diff"));

    Ok(())
}

#[tokio::test]
async fn test_merge_instance_configs_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let source_container = SyncThingContainer::new().await?;
    let dest_container = SyncThingContainer::new().await?;

    let source_config = source_container.instance_config();
    let dest_config = dest_container.instance_config();

    let mut app_config = syncthing_mcp_rs::config::AppConfig {
        instances: vec![
            syncthing_mcp_rs::config::InstanceConfig {
                name: Some("source".to_string()),
                ..source_config
            },
            syncthing_mcp_rs::config::InstanceConfig {
                name: Some("dest".to_string()),
                ..dest_config
            },
        ],
        ..Default::default()
    };
    app_config.validate().await.unwrap();

    let source_client = source_container.client();
    let dest_client = dest_container.client();

    // 1. Add a folder to source
    source_client
        .add_folder("folder-source", "Source Folder", "/tmp/source")
        .await?;

    // 2. Add a folder to dest (should be preserved)
    dest_client
        .add_folder("folder-dest", "Dest Folder", "/tmp/dest")
        .await?;

    let ctx = TestContext::from_container(source_container);
    let mut ctx = ctx;
    ctx.config = app_config;

    // 3. Call merge_instance_configs
    let result = ctx
        .call_tool(
            "merge_instance_configs",
            json!({
                "source": "source",
                "destination": "dest"
            }),
        )
        .await?;

    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Successfully merged configuration"));
    assert!(text.contains("+ Folder: folder-source"));

    // 4. Verify on destination
    let dest_folders = dest_client.list_folders().await?;
    assert!(dest_folders.iter().any(|f| f.id == "folder-source"));
    assert!(dest_folders.iter().any(|f| f.id == "folder-dest"));

    Ok(())
}

#[tokio::test]
async fn test_tool_error_reporting() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;
    let result = ctx
        .call_tool(
            "manage_folders",
            json!({
                "action": "get",
                "folder_id": "non-existent-folder"
            }),
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let resp_err = err
        .downcast_ref::<syncthing_mcp_rs::error::Error>()
        .unwrap();

    let diagnostic = resp_err.diagnose();
    assert!(diagnostic.advice.contains("Verify the ID and endpoint"));

    match resp_err {
        syncthing_mcp_rs::error::Error::NotFound(_) => {
            // Expected error
        }
        _ => panic!("Expected NotFound error, but got {:?}", resp_err),
    }

    Ok(())
}

#[tokio::test]
async fn test_conflict_diff_and_preview_tools() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    use tempfile::tempdir;
    let temp_dir = tempdir()?;
    let host_path = temp_dir.path().to_str().unwrap().to_string();
    let container_path = "/var/syncthing/test-mount".to_string();

    let container =
        SyncThingContainer::with_mount(Some((host_path.clone(), container_path.clone()))).await?;
    let ctx = TestContext::from_container(container);

    // Create original and conflict files on the host (accessible to both)
    let original_filename = "test.txt";
    let conflict_filename = "test.sync-conflict-20230101-120000-DEVICE.txt";

    let host_original_path = temp_dir.path().join(original_filename);
    let host_conflict_path = temp_dir.path().join(conflict_filename);

    std::fs::write(&host_original_path, "original content")?;
    std::fs::write(&host_conflict_path, "conflict content")?;

    // The tool will be called with the path as seen by the MCP server (host path)
    let mcp_conflict_path = host_conflict_path.to_str().unwrap();

    // 1. Test diff_conflicts
    let result = ctx
        .call_tool(
            "diff_conflicts",
            json!({
                "conflict_path": mcp_conflict_path
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("-original content"));
    assert!(text.contains("+conflict content"));

    // 2. Test preview_conflict_resolution
    let result = ctx
        .call_tool(
            "preview_conflict_resolution",
            json!({
                "conflict_path": mcp_conflict_path,
                "action": "keep_conflict"
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert_eq!(text, "conflict content");

    Ok(())
}

#[tokio::test]
async fn test_bandwidth_tools() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;

    // 1. Test set_bandwidth_limits
    let result = ctx
        .call_tool(
            "set_bandwidth_limits",
            json!({
                "max_recv_kbps": 1234,
                "max_send_kbps": 5678
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Bandwidth limits updated"));

    // 2. Test get_bandwidth_status
    let result = ctx.call_tool("get_bandwidth_status", json!({})).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("1234"));
    assert!(text.contains("5678"));

    // 3. Test set_performance_profile
    // We need to add a profile to the config first
    let mut ctx = ctx;
    ctx.config
        .bandwidth
        .profiles
        .push(syncthing_mcp_rs::config::PerformanceProfile {
            name: "test_profile".to_string(),
            limits: syncthing_mcp_rs::config::BandwidthLimits {
                max_recv_kbps: Some(9999),
                max_send_kbps: Some(8888),
            },
        });

    let result = ctx
        .call_tool(
            "set_performance_profile",
            json!({
                "name": "test_profile"
            }),
        )
        .await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("test_profile' applied"));

    // 4. Verify limits were applied from profile
    let result = ctx.call_tool("get_bandwidth_status", json!({})).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("9999"));
    assert!(text.contains("8888"));

    Ok(())
}
