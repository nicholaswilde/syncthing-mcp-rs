#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use crate::mcp::server::McpServer;
    use crate::tools::create_registry;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_tool_call_list_conflicts_success() {
        let mock_server = MockServer::start().await;
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        // Create a conflict file
        let conflict_file = folder_path.join("notes.sync-conflict-20230101-120000-ABCDEFG.txt");
        fs::write(&conflict_file, "conflict content").unwrap();
        fs::write(folder_path.join("notes.txt"), "original content").unwrap();

        Mock::given(method("GET"))
            .and(path("/rest/config/folders/default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "default",
                "label": "Default Folder",
                "path": folder_path.to_str().unwrap(),
                "type": "sendreceive",
                "devices": [],
                "rescan_interval_s": 3600,
                "fs_watcher_enabled": true,
                "paused": false
            })))
            .mount(&mock_server)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "list_conflicts",
                "arguments": {
                    "folder_id": "default"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Conflicts in folder default:"));
        assert!(text.contains("notes.sync-conflict-20230101-120000-ABCDEFG.txt"));
        assert!(text.contains("Size:"));
        assert!(text.contains("Modified:"));
    }

    #[tokio::test]
    async fn test_tool_call_list_conflicts_empty() {
        let mock_server = MockServer::start().await;
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        Mock::given(method("GET"))
            .and(path("/rest/config/folders/default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "default",
                "label": "Default Folder",
                "path": folder_path.to_str().unwrap(),
                "type": "sendreceive",
                "devices": [],
                "rescan_interval_s": 3600,
                "fs_watcher_enabled": true,
                "paused": false
            })))
            .mount(&mock_server)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "list_conflicts",
                "arguments": {
                    "folder_id": "default"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("No conflicts found in folder default."));
    }

    #[tokio::test]
    async fn test_tool_call_resolve_conflict_keep_original() {
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        let original_file = folder_path.join("notes.txt");
        let conflict_file = folder_path.join("notes.sync-conflict-20230101-120000-ABCDEFG.txt");
        fs::write(&original_file, "original content").unwrap();
        fs::write(&conflict_file, "conflict content").unwrap();

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: "http://localhost".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "resolve_conflict",
                "arguments": {
                    "conflict_path": conflict_file.to_str().unwrap(),
                    "action": "keep_original",
                    "backup": false
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Resolved conflict by keeping original version"));

        assert!(original_file.exists());
        assert!(!conflict_file.exists());
        assert_eq!(
            fs::read_to_string(&original_file).unwrap(),
            "original content"
        );
    }

    #[tokio::test]
    async fn test_tool_call_resolve_conflict_keep_conflict() {
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        let original_file = folder_path.join("notes.txt");
        let conflict_file = folder_path.join("notes.sync-conflict-20230101-120000-ABCDEFG.txt");
        fs::write(&original_file, "original content").unwrap();
        fs::write(&conflict_file, "conflict content").unwrap();

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: "http://localhost".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "resolve_conflict",
                "arguments": {
                    "conflict_path": conflict_file.to_str().unwrap(),
                    "action": "keep_conflict",
                    "backup": false
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Resolved conflict by keeping conflict version"));

        assert!(original_file.exists());
        assert!(!conflict_file.exists());
        assert_eq!(
            fs::read_to_string(&original_file).unwrap(),
            "conflict content"
        );
    }

    #[tokio::test]
    async fn test_tool_call_delete_conflict() {
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        let conflict_file = folder_path.join("notes.sync-conflict-20230101-120000-ABCDEFG.txt");
        fs::write(&conflict_file, "conflict content").unwrap();

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: "http://localhost".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "delete_conflict",
                "arguments": {
                    "conflict_path": conflict_file.to_str().unwrap()
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Moved conflict file to trash:"));

        assert!(!conflict_file.exists());
    }

    #[tokio::test]
    async fn test_tool_call_resolve_conflict_dry_run() {
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        let original_file = folder_path.join("notes.txt");
        let conflict_file = folder_path.join("notes.sync-conflict-20230101-120000-ABCDEFG.txt");
        fs::write(&original_file, "original content").unwrap();
        fs::write(&conflict_file, "conflict content").unwrap();

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: "http://localhost".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "resolve_conflict",
                "arguments": {
                    "conflict_path": conflict_file.to_str().unwrap(),
                    "action": "keep_conflict",
                    "dry_run": true
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("[DRY RUN] Would resolve conflict by keeping conflict version"));

        // Files should still exist as they were
        assert!(original_file.exists());
        assert!(conflict_file.exists());
        assert_eq!(
            fs::read_to_string(&original_file).unwrap(),
            "original content"
        );
    }

    #[tokio::test]
    async fn test_tool_call_delete_conflict_dry_run() {
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        let conflict_file = folder_path.join("notes.sync-conflict-20230101-120000-ABCDEFG.txt");
        fs::write(&conflict_file, "conflict content").unwrap();

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: "http://localhost".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "delete_conflict",
                "arguments": {
                    "conflict_path": conflict_file.to_str().unwrap(),
                    "dry_run": true
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("[DRY RUN] Would delete conflict file:"));

        assert!(conflict_file.exists());
    }

    #[tokio::test]
    async fn test_tool_call_resolve_conflict_with_backup() {
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        let original_file = folder_path.join("notes.txt");
        let conflict_file = folder_path.join("notes.sync-conflict-20230101-120000-ABCDEFG.txt");
        fs::write(&original_file, "original content").unwrap();
        fs::write(&conflict_file, "conflict content").unwrap();

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: "http://localhost".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "resolve_conflict",
                "arguments": {
                    "conflict_path": conflict_file.to_str().unwrap(),
                    "action": "keep_original",
                    "backup": true
                }
            })),
        };

        // This might fail in CI if trash is not available, but we'll try to pass it
        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();

        // If it succeeds, it should mention backup
        assert!(text.contains("moved") && text.contains("to trash"));

        assert!(original_file.exists());
        assert!(!conflict_file.exists());
    }

    #[tokio::test]
    async fn test_tool_call_list_conflicts_recursive() {
        let mock_server = MockServer::start().await;
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        let sub_dir = folder_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let conflict_file = sub_dir.join("notes.sync-conflict-20230101-120000-ABCDEFG.txt");
        fs::write(&conflict_file, "conflict content").unwrap();

        Mock::given(method("GET"))
            .and(path("/rest/config/folders/default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "default",
                "label": "Default Folder",
                "path": folder_path.to_str().unwrap(),
                "type": "sendreceive",
                "devices": [],
                "rescan_interval_s": 3600,
                "fs_watcher_enabled": true,
                "paused": false
            })))
            .mount(&mock_server)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "list_conflicts",
                "arguments": {
                    "folder_id": "default"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("notes.sync-conflict-20230101-120000-ABCDEFG.txt"));
    }

    #[tokio::test]
    async fn test_tool_call_list_conflicts_inaccessible() {
        let mock_server = MockServer::start().await;
        let temp_dir = tempdir().unwrap();
        let folder_path = temp_dir.path();

        let sub_dir = folder_path.join("inaccessible");
        fs::create_dir(&sub_dir).unwrap();

        // Make it inaccessible
        let mut perms = fs::metadata(&sub_dir).unwrap().permissions();
        perms.set_readonly(true); // This doesn't necessarily make it unreadable on Linux, but let's try

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o000);
            fs::set_permissions(&sub_dir, perms).unwrap();
        }

        Mock::given(method("GET"))
            .and(path("/rest/config/folders/default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "default",
                "label": "Default Folder",
                "path": folder_path.to_str().unwrap(),
                "type": "sendreceive",
                "devices": [],
                "rescan_interval_s": 3600,
                "fs_watcher_enabled": true,
                "paused": false
            })))
            .mount(&mock_server)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("default".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "list_conflicts",
                "arguments": {
                    "folder_id": "default"
                }
            })),
        };

        let resp = server.handle_request(req).await;

        // Cleanup permissions so temp_dir can be deleted
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&sub_dir).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&sub_dir, perms).unwrap();
        }

        assert!(resp.is_err());
        assert!(
            resp.unwrap_err()
                .to_string()
                .contains("Failed to read directory")
        );
    }
}
