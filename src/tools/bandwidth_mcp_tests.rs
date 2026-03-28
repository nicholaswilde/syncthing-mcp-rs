#[cfg(test)]
mod tests {
    use crate::config::{AppConfig, InstanceConfig};
    use crate::mcp::server::McpServer;
    use crate::tools::create_registry;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_tool_call_set_bandwidth_limits() {
        let mock_server = MockServer::start().await;

        let initial_config = json!({
            "version": 37,
            "folders": [],
            "devices": [],
            "gui": {},
            "ldap": {},
            "options": {
                "maxRecvKbps": 0,
                "maxSendKbps": 0
            },
            "remoteIgnoredDevices": [],
            "defaults": {},
            "pendingDevices": {}
        });

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(initial_config.clone()))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("test-instance".to_string()),
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
                "name": "set_bandwidth_limits",
                "arguments": {
                    "max_recv_kbps": 1000,
                    "max_send_kbps": 500
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Bandwidth limits updated successfully"));
    }

    #[tokio::test]
    async fn test_tool_call_set_performance_profile() {
        let mock_server = MockServer::start().await;

        let initial_config = json!({
            "version": 37,
            "folders": [],
            "devices": [],
            "gui": {},
            "ldap": {},
            "options": {
                "maxRecvKbps": 0,
                "maxSendKbps": 0
            },
            "remoteIgnoredDevices": [],
            "defaults": {}
        });

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(initial_config.clone()))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("test-instance".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            bandwidth: crate::config::BandwidthConfig {
                profiles: vec![crate::config::PerformanceProfile {
                    name: "working_hours".to_string(),
                    limits: crate::config::BandwidthLimits {
                        max_recv_kbps: Some(1000),
                        max_send_kbps: Some(500),
                    },
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);

        let req = crate::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: crate::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "set_performance_profile",
                "arguments": {
                    "name": "working_hours"
                }
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Performance profile 'working_hours' applied successfully"));
    }

    #[tokio::test]
    async fn test_tool_call_get_bandwidth_status() {
        let mock_server = MockServer::start().await;

        let config_data = json!({
            "version": 37,
            "folders": [],
            "devices": [],
            "gui": {},
            "ldap": {},
            "options": {
                "maxRecvKbps": 1000,
                "maxSendKbps": 500
            },
            "remoteIgnoredDevices": [],
            "defaults": {}
        });

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(config_data))
            .mount(&mock_server)
            .await;

        let registry = create_registry();
        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("test-instance".to_string()),
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
                "name": "get_bandwidth_status",
                "arguments": {}
            })),
        };

        let resp = server.handle_request(req).await.unwrap();
        let text = resp["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("- Instance test-instance: Recv 1000 Kbps, Send 500 Kbps"));
    }
}
