#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use crate::mcp::server::McpServer;
    use crate::tools::create_registry;
    use serde_json::json;

    #[tokio::test]
    async fn test_initialize() {
        let registry = create_registry();
        let config = AppConfig::default();
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "initialize".to_string(),
            params: None,
        };

        let resp = server.handle_request(req).await.unwrap();
        assert_eq!(resp["serverInfo"]["name"], "syncthing-mcp-rs");
    }

    #[tokio::test]
    async fn test_tools_list() {
        let registry = create_registry();
        let config = AppConfig::default();
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let resp = server.handle_request(req).await.unwrap();
        let tools = resp["tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t["name"] == "get_system_stats"));
        assert!(tools.iter().any(|t| t["name"] == "manage_folders"));
    }

    #[tokio::test]
    async fn test_tool_call_get_system_stats() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "myID": "test-id",
                "uptime": 100,
                "alloc": 1000,
                "sys": 2000,
                "goroutines": 10,
                "pathSeparator": "/"
            })))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/system/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": "v1.0.0",
                "arch": "amd64",
                "os": "linux",
                "isRelease": true,
                "isBeta": false,
                "isCandidate": false
            })))
            .mount(&mock_server)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            host: "localhost".to_string(),
            port: 8384,
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
                "name": "get_system_stats",
                "arguments": {}
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Version: v1.0.0"));
        assert!(text.contains("My ID: test-id"));
    }

    #[tokio::test]
    async fn test_tool_call_manage_folders() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/config/folders"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "default",
                    "label": "Default Folder",
                    "path": "/home/sync",
                    "type": "sendreceive",
                    "devices": [],
                    "rescan_interval_s": 3600,
                    "fs_watcher_enabled": true,
                    "paused": false
                }
            ])))
            .mount(&mock_server)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            host: "localhost".to_string(),
            port: 8384,
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
                "name": "manage_folders",
                "arguments": {
                    "action": "list"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Folders:"));
        assert!(text.contains("- Default Folder (default): /home/sync (paused: false)"));
    }

    #[tokio::test]
    async fn test_run_stdio_shutdown() {
        let registry = create_registry();
        let config = AppConfig::default();
        let (server, rx) = McpServer::new(registry, config);

        let input = b""; // Empty input should cause loop to break
        let mut output = Vec::new();

        server.run(&input[..], &mut output, rx).await.unwrap();
        assert!(output.is_empty());
    }
}
