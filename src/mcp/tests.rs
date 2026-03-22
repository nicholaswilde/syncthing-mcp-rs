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
    async fn test_tool_call_configure_sharing() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock GET folder
        Mock::given(method("GET"))
            .and(path("/rest/config/folders/default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "default",
                "label": "Default Folder",
                "path": "/home/sync",
                "type": "sendreceive",
                "devices": [],
                "rescan_interval_s": 3600,
                "fs_watcher_enabled": true,
                "paused": false
            })))
            .mount(&mock_server)
            .await;

        // Mock PATCH folder
        Mock::given(method("PATCH"))
            .and(path("/rest/config/folders/default"))
            .respond_with(ResponseTemplate::new(200))
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
                "name": "configure_sharing",
                "arguments": {
                    "action": "share",
                    "folder_id": "default",
                    "device_id": "device1"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Folder default shared with device device1 successfully"));
    }

    #[tokio::test]
    async fn test_tool_call_manage_ignores() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/db/ignores"))
            .and(query_param("folder", "default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "ignore": ["node_modules"],
                "expanded": ["node_modules"]
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
                "name": "manage_ignores",
                "arguments": {
                    "action": "get",
                    "folder_id": "default"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Ignore patterns for folder default:"));
        assert!(text.contains("- node_modules"));
    }

    #[tokio::test]
    async fn test_tool_call_manage_devices() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/config/devices"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "deviceID": "test-device-id",
                    "name": "Test Device",
                    "addresses": ["dynamic"],
                    "compression": "metadata",
                    "introducer": false,
                    "paused": false,
                    "untrusted": false
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
                "name": "manage_devices",
                "arguments": {
                    "action": "list"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Devices:"));
        assert!(text.contains("- Test Device (test-device-id): (paused: false)"));
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

    #[tokio::test]
    async fn test_run_with_request() {
        let registry = create_registry();
        let config = AppConfig::default();
        let (server, rx) = McpServer::new(registry, config);

        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        });
        let input = serde_json::to_vec(&req).unwrap();
        let mut input_with_newline = input.clone();
        input_with_newline.push(b'\n');

        let mut output = Vec::new();
        server.run(&input_with_newline[..], &mut output, rx).await.unwrap();

        let resp: crate::mcp::Response = serde_json::from_slice(&output).unwrap();
        assert_eq!(resp.id, crate::mcp::RequestId::Number(1));
    }

    #[tokio::test]
    async fn test_run_with_notification() {
        let registry = create_registry();
        let config = AppConfig::default();
        let (server, rx) = McpServer::new(registry, config);

        let (client_writer, server_reader) = tokio::io::duplex(1024);
        let (server_writer, mut client_reader) = tokio::io::duplex(1024);

        let tx = server.notification_tx.clone();
        tokio::spawn(async move {
            tx.send(crate::mcp::Notification {
                jsonrpc: "2.0".to_string(),
                method: "test/notify".to_string(),
                params: None,
            })
            .await
            .unwrap();
            
            // Give some time for notification to be processed
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            // Close the writer to exit the loop
            drop(client_writer);
        });

        server.run(server_reader, server_writer, rx).await.unwrap();

        let mut output = Vec::new();
        tokio::io::copy(&mut client_reader, &mut output).await.unwrap();

        let msg: crate::mcp::Message = serde_json::from_slice(&output).unwrap();
        if let crate::mcp::Message::Notification(n) = msg {
            assert_eq!(n.method, "test/notify");
        } else {
            panic!("Expected notification");
        }
    }

    #[tokio::test]
    async fn test_unknown_tool() {
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
                "name": "unknown_tool",
                "arguments": {}
            })),
        };

        let resp = server.handle_request(req).await;
        assert!(resp.is_err());
    }
}
