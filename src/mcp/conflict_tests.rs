#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use crate::mcp::server::McpServer;
    use crate::tools::create_registry;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use std::fs;
    use tempfile::tempdir;

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
}
