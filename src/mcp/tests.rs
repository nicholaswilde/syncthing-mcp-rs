#[cfg(test)]
#[allow(clippy::module_inception)]
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
    async fn test_tool_call_manage_devices_add() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/rest/config/devices"))
            .respond_with(ResponseTemplate::new(200))
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
                "name": "manage_devices",
                "arguments": {
                    "action": "add",
                    "device_id": "new-device-id",
                    "name": "New Device"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Device new-device-id added successfully"));
    }

    #[tokio::test]
    async fn test_tool_call_manage_devices_remove() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/rest/config/devices/test-device-id"))
            .respond_with(ResponseTemplate::new(200))
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
                "name": "manage_devices",
                "arguments": {
                    "action": "remove",
                    "device_id": "test-device-id"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Device test-device-id removed successfully"));
    }

    #[tokio::test]
    async fn test_tool_call_manage_devices_pause_resume() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/rest/config/devices/test-device-id"))
            .respond_with(ResponseTemplate::new(200))
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

        // Test pause
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "manage_devices",
                "arguments": {
                    "action": "pause",
                    "device_id": "test-device-id"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Device test-device-id paused successfully"));

        // Test resume
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(2),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "manage_devices",
                "arguments": {
                    "action": "resume",
                    "device_id": "test-device-id"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Device test-device-id resumed successfully"));
    }

    #[tokio::test]
    async fn test_tool_call_configure_sharing_unshare() {
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
                "devices": [{"deviceID": "device1"}],
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
                    "action": "unshare",
                    "folder_id": "default",
                    "device_id": "device1"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Folder default unshared from device device1 successfully"));
    }

    #[tokio::test]
    async fn test_tool_call_manage_ignores_set_append() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock POST ignores (for set)
        Mock::given(method("POST"))
            .and(path("/rest/db/ignores"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        // Mock GET ignores (for append)
        Mock::given(method("GET"))
            .and(path("/rest/db/ignores"))
            .and(query_param("folder", "default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "ignore": ["node_modules"]
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

        // Test set
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "manage_ignores",
                "arguments": {
                    "action": "set",
                    "folder_id": "default",
                    "patterns": ["temp", "bin"]
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully set 2 ignore patterns for folder default"));

        // Test append
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(2),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "manage_ignores",
                "arguments": {
                    "action": "append",
                    "folder_id": "default",
                    "patterns": ["build"]
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully appended 1 new ignore patterns to folder default"));
    }

    #[tokio::test]
    async fn test_tool_call_maintain_system() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/rest/db/scan"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/rest/system/restart"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/rest/system/error/clear"))
            .respond_with(ResponseTemplate::new(200))
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

        // Test force_rescan
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "maintain_system",
                "arguments": {
                    "action": "force_rescan",
                    "folder_id": "default"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully triggered rescan for folder: default"));

        // Test restart
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(2),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "maintain_system",
                "arguments": {
                    "action": "restart"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully triggered SyncThing restart"));

        // Test clear_errors
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(3),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "maintain_system",
                "arguments": {
                    "action": "clear_errors"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully cleared SyncThing errors"));
    }

    #[tokio::test]
    async fn test_tool_call_edge_cases() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

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

        // 1. manage_devices: unnamed device
        Mock::given(method("GET"))
            .and(path("/rest/config/devices"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "deviceID": "test-device-id",
                    "addresses": ["dynamic"],
                    "compression": "metadata",
                    "introducer": false,
                    "paused": false,
                    "untrusted": false
                }
            ])))
            .mount(&mock_server)
            .await;

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "manage_devices",
                "arguments": { "action": "list" }
            })),
        };
        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("- unnamed (test-device-id)"));

        // 2. manage_devices: missing device_id for add
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(2),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "manage_devices",
                "arguments": { "action": "add" }
            })),
        };
        let resp = server.handle_request(req).await;
        assert!(resp.is_err());
        assert!(
            resp.unwrap_err()
                .to_string()
                .contains("device_id is required")
        );

        // 3. manage_devices: unsupported action
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(3),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "manage_devices",
                "arguments": { "action": "invalid" }
            })),
        };
        let resp = server.handle_request(req).await;
        assert!(resp.is_err());
        assert!(resp.unwrap_err().to_string().contains("Unsupported action"));

        // 4. configure_sharing: unshare when not shared
        Mock::given(method("GET"))
            .and(path("/rest/config/folders/default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "default", "label": "Default", "path": "/sync", "type": "sendreceive",
                "devices": [], "rescan_interval_s": 3600, "fs_watcher_enabled": true, "paused": false
            })))
            .mount(&mock_server)
            .await;

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(4),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "configure_sharing",
                "arguments": { "action": "unshare", "folder_id": "default", "device_id": "device1" }
            })),
        };
        let resp = server.handle_request(req).await.unwrap();
        assert!(
            resp["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("unshared")
        );

        // 5. manage_ignores: missing patterns for set
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(5),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "manage_ignores",
                "arguments": { "action": "set", "folder_id": "default" }
            })),
        };
        let resp = server.handle_request(req).await;
        assert!(resp.is_err());
        assert!(
            resp.unwrap_err()
                .to_string()
                .contains("patterns array is required")
        );

        // 6. manage_ignores: append no new patterns
        Mock::given(method("GET"))
            .and(path("/rest/db/ignores"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ignore": ["a"] })))
            .mount(&mock_server)
            .await;

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(6),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "manage_ignores",
                "arguments": { "action": "append", "folder_id": "default", "patterns": ["a"] }
            })),
        };
        let resp = server.handle_request(req).await.unwrap();
        assert!(
            resp["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Successfully appended 0")
        );

        // 7. maintain_system: invalid action
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(7),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "maintain_system",
                "arguments": { "action": "invalid" }
            })),
        };
        let resp = server.handle_request(req).await;
        assert!(resp.is_err());
        assert!(resp.unwrap_err().to_string().contains("Invalid action"));
    }

    #[tokio::test]
    async fn test_tool_call_with_instance_selection() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server1 = MockServer::start().await;
        let mock_server2 = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "myID": "id1", "uptime": 100, "alloc": 1000, "sys": 2000, "goroutines": 10, "pathSeparator": "/"
            })))
            .mount(&mock_server1)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "myID": "id2", "uptime": 200, "alloc": 2000, "sys": 4000, "goroutines": 20, "pathSeparator": "/"
            })))
            .mount(&mock_server2)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/system/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": "v1", "arch": "amd64", "os": "linux", "isRelease": true, "isBeta": false, "isCandidate": false
            })))
            .mount(&mock_server1)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/system/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": "v2", "arch": "amd64", "os": "linux", "isRelease": true, "isBeta": false, "isCandidate": false
            })))
            .mount(&mock_server2)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![
                crate::config::InstanceConfig {
                    name: Some("inst1".to_string()),
                    url: mock_server1.uri(),
                    ..Default::default()
                },
                crate::config::InstanceConfig {
                    name: Some("inst2".to_string()),
                    url: mock_server2.uri(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        // Test with instance name "inst2"
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "get_system_stats",
                "arguments": {
                    "instance": "inst2"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("My ID: id2"));

        // Test with instance index "0"
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(2),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "get_system_stats",
                "arguments": {
                    "instance": "0"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("My ID: id1"));
    }

    #[tokio::test]
    async fn test_tool_call_errors() {
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

        // Test non-existent instance
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "get_system_stats",
                "arguments": {
                    "instance": "non-existent"
                }
            })),
        };
        let resp = server.handle_request(req).await;
        assert!(resp.is_err());
        assert!(resp.unwrap_err().to_string().contains("Instance not found"));

        // Test missing tool name
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(2),
            method: "tools/call".to_string(),
            params: Some(json!({
                "arguments": {}
            })),
        };
        let resp = server.handle_request(req).await;
        assert!(resp.is_err());

        // Test unknown method
        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(3),
            method: "unknown/method".to_string(),
            params: None,
        };
        let resp = server.handle_request(req).await;
        assert!(resp.is_err());
        assert!(resp.unwrap_err().to_string().contains("Method not found"));
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
        server
            .run(&input_with_newline[..], &mut output, rx)
            .await
            .unwrap();

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
        tokio::io::copy(&mut client_reader, &mut output)
            .await
            .unwrap();

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

    #[tokio::test]
    async fn test_tool_call_get_sync_status_folder() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/db/status"))
            .and(query_param("folder", "default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "state": "idle",
                "needBytes": 0,
                "needFiles": 0,
                "inSyncBytes": 1000,
                "inSyncFiles": 10,
                "globalBytes": 1000,
                "globalFiles": 10,
                "localBytes": 1000,
                "localFiles": 10
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
                "name": "get_sync_status",
                "arguments": {
                    "target": "folder",
                    "id": "default"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Folder: default"));
        assert!(text.contains("Completion: 100.00%"));
    }

    #[tokio::test]
    async fn test_tool_call_get_sync_status_device() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/db/completion"))
            .and(query_param("device", "device1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "completion": 50.5,
                "needBytes": 500,
                "needFiles": 5,
                "globalBytes": 1000
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
                "name": "get_sync_status",
                "arguments": {
                    "target": "device",
                    "id": "device1"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Device: device1"));
        assert!(text.contains("Completion: 50.50%"));
        assert!(text.contains("Bytes Remaining: 500"));
    }

    #[tokio::test]
    async fn test_tool_call_replicate_config() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        // Source config
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": 37,
                "folders": [{"id": "folder1"}],
                "devices": [{"deviceID": "device1"}]
            })))
            .mount(&source_mock)
            .await;

        // Destination config (get)
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": 37,
                "folders": [],
                "devices": []
            })))
            .mount(&dest_mock)
            .await;

        // Destination config (set)
        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&dest_mock)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![
                crate::config::InstanceConfig {
                    name: Some("source".to_string()),
                    url: source_mock.uri(),
                    ..Default::default()
                },
                crate::config::InstanceConfig {
                    name: Some("dest".to_string()),
                    url: dest_mock.uri(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "replicate_config",
                "arguments": {
                    "source": "source",
                    "destination": "dest"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully replicated configuration to dest"));
        assert!(text.contains("Folders: 1 added, 0 removed, 0 updated."));
        assert!(text.contains("Devices: 1 added, 0 removed, 0 updated."));
    }

    #[tokio::test]
    async fn test_tool_call_get_system_connections() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/system/connections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "DEVICE-ID-1": {
                    "at": "2023-10-24T12:34:56Z",
                    "inBytesTotal": 1000,
                    "outBytesTotal": 2000,
                    "address": "1.2.3.4:22000",
                    "clientVersion": "v1.27.0",
                    "connected": true,
                    "type": "tcp-client",
                    "isPaused": false
                }
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
                "name": "get_system_connections",
                "arguments": {}
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Connection Status:"));
        assert!(text.contains("Device: DEVICE-ID-1"));
        assert!(text.contains("Connected: true"));
        assert!(text.contains("In Bytes: 1000"));
    }

    #[tokio::test]
    async fn test_tool_call_get_system_log() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/system/log"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "messages": [
                    {
                        "when": "2023-10-27T10:00:00Z",
                        "message": "test log message"
                    }
                ]
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
                "name": "get_system_log",
                "arguments": {}
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing System Log:"));
        assert!(text.contains("[2023-10-27T10:00:00Z] test log message"));
    }

    #[tokio::test]
    async fn test_tool_call_get_device_stats() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/stats/device"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "DEVICE-ID-1": {
                    "lastSeen": "2023-10-27T15:33:10Z",
                    "lastConnectionDurationS": 3600.5
                }
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
                "name": "get_device_stats",
                "arguments": {}
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Device Statistics:"));
        assert!(text.contains("Device: DEVICE-ID-1"));
        assert!(text.contains("Last Seen: 2023-10-27T15:33:10Z"));
    }

    #[tokio::test]
    async fn test_tool_call_get_folder_stats() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/stats/folder"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folder1": {
                    "lastScan": "2023-10-27T14:20:01Z",
                    "lastFile": {
                        "filename": "test.txt",
                        "at": "2023-10-27T14:19:55Z"
                    }
                }
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
                "name": "get_folder_stats",
                "arguments": {}
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Folder Statistics:"));
        assert!(text.contains("Folder: folder1"));
        assert!(text.contains("Last Scan: 2023-10-27T14:20:01Z"));
        assert!(text.contains("Filename: test.txt"));
    }

    #[tokio::test]
    async fn test_tool_call_manage_folders_pending() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/cluster/pending/folders"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folder1": {
                    "offeredBy": {
                        "DEVICE1": {
                            "time": "2023-10-27T10:00:00Z",
                            "label": "Test Folder",
                            "receiveEncrypted": false,
                            "remoteEncrypted": false
                        }
                    }
                }
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
                "name": "manage_folders",
                "arguments": {
                    "action": "pending"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Pending Folder Requests:"));
        assert!(text.contains("- folder1 (folder1)"));
        assert!(text.contains("Offered by: DEVICE1 (label: Test Folder, time: 2023-10-27T10:00:00Z)"));
    }

    #[tokio::test]
    async fn test_tool_call_manage_folders_revert() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/rest/db/revert"))
            .and(query_param("folder", "default"))
            .respond_with(ResponseTemplate::new(200))
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
                "name": "manage_folders",
                "arguments": {
                    "action": "revert",
                    "folder_id": "default"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully triggered revert for folder: default"));
    }
}
