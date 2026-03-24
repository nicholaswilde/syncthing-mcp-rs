#[cfg(test)]
mod tests {
    use crate::config::{AppConfig, InstanceConfig};
    use crate::mcp::{McpServer, Request, RequestId};
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_mcp_error_unauthorized() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("default".to_string()),
                url: mock_server.uri(),
                api_key: Some("wrong-key".to_string()),
                retry_max_attempts: Some(1),
                ..Default::default()
            }],
            ..Default::default()
        };

        let registry = crate::tools::create_registry();
        let (server, _rx) = McpServer::new(registry, config);

        let req = Request {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "get_system_status",
                "arguments": {}
            })),
        };

        let response = server.handle_request(req).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        let mcp_err = crate::mcp::ResponseError::from(err);

        assert_eq!(mcp_err.code, -32001);
        assert!(mcp_err.message.contains("Unauthorized"));
    }

    #[tokio::test]
    async fn test_mcp_error_network() {
        // No mock server listening on this port
        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("default".to_string()),
                url: "http://127.0.0.1:1".to_string(),
                retry_max_attempts: Some(1),
                retry_initial_backoff_ms: Some(1),
                ..Default::default()
            }],
            ..Default::default()
        };

        let registry = crate::tools::create_registry();
        let (server, _rx) = McpServer::new(registry, config);

        let req = Request {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "get_system_status",
                "arguments": {}
            })),
        };

        let response = server.handle_request(req).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        let mcp_err = crate::mcp::ResponseError::from(err);

        assert_eq!(mcp_err.code, -32004);
        assert!(mcp_err.message.contains("Network Error"));
    }
}
