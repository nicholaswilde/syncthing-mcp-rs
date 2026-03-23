#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use crate::mcp::server::McpServer;
    use crate::tools::ToolRegistry;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_sse_endpoint_establishes_session() {
        let registry = ToolRegistry::new();
        let config = AppConfig::default();
        let (server, _rx) = McpServer::new(registry, config);
        let app = server.router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/event-stream");

        let mut body = response.into_body().into_data_stream();
        let first_event = body.next().await.unwrap().unwrap();
        let event_str = String::from_utf8(first_event.to_vec()).unwrap();
        
        // MCP SSE spec says the first event should be 'endpoint' containing the URL for POSTing messages
        assert!(event_str.contains("event: endpoint"));
        assert!(event_str.contains("data: /message?session_id="));
    }

    #[tokio::test]
    async fn test_message_endpoint_routing() {
        let registry = ToolRegistry::new();
        let config = AppConfig::default();
        let (server, _rx) = McpServer::new(registry, config);
        let app = server.router();

        // 1. Establish session
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .uri("/sse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        let mut body = response.into_body().into_data_stream();
        let first_event = body.next().await.unwrap().unwrap();
        let event_str = String::from_utf8(first_event.to_vec()).unwrap();
        
        // Extract session ID from endpoint data: "/message?session_id=UUID"
        let session_id = event_str.split("session_id=").collect::<Vec<_>>()[1].trim();

        // 2. Post a request
        let mcp_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/message?session_id={}", session_id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&mcp_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body_bytes = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
        let mcp_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        
        assert_eq!(mcp_resp["id"], 1);
        assert_eq!(mcp_resp["result"]["serverInfo"]["name"], "syncthing-mcp-rs");
    }

    #[tokio::test]
    async fn test_tools_list_over_http() {
        let registry = ToolRegistry::new();
        let config = AppConfig::default();
        let (server, _rx) = McpServer::new(registry, config);
        let app = server.router();

        // 1. Establish session
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .uri("/sse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        let mut body = response.into_body().into_data_stream();
        let first_event = body.next().await.unwrap().unwrap();
        let event_str = String::from_utf8(first_event.to_vec()).unwrap();
        let session_id = event_str.split("session_id=").collect::<Vec<_>>()[1].trim();

        // 2. Request tools/list
        let mcp_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "list-req",
            "method": "tools/list",
            "params": {}
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/message?session_id={}", session_id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&mcp_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body_bytes = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
        let mcp_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        
        assert_eq!(mcp_resp["id"], "list-req");
        assert!(mcp_resp["result"]["tools"].is_array());
    }

    #[tokio::test]
    async fn test_http_auth_failure() {
        let registry = ToolRegistry::new();
        let mut config = AppConfig::default();
        config.http_server.api_key = Some("secret-token".to_string());
        let (server, _rx) = McpServer::new(registry, config);
        let app = server.router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_http_auth_success() {
        let registry = ToolRegistry::new();
        let mut config = AppConfig::default();
        config.http_server.api_key = Some("secret-token".to_string());
        let (server, _rx) = McpServer::new(registry, config);
        let app = server.router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sse")
                    .header("authorization", "Bearer secret-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_sse_notifications() {
        let registry = ToolRegistry::new();
        let config = AppConfig::default();
        let (server, mut rx) = McpServer::new(registry, config);
        let app = server.router();

        // 1. Establish SSE session
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        let mut body = response.into_body().into_data_stream();
        
        // First event is 'endpoint'
        let _endpoint_event = body.next().await.unwrap().unwrap();

        // 2. Trigger a notification
        let notification = crate::mcp::Notification {
            jsonrpc: "2.0".to_string(),
            method: "test/notification".to_string(),
            params: Some(serde_json::json!({"foo": "bar"})),
        };
        
        // In the real app, the run loop or event loop handles this.
        // We need to simulate the notification being sent to all sessions.
        let n_clone = notification.clone();
        let sessions = server.sessions.clone();
        tokio::spawn(async move {
            for session in sessions.iter() {
                let _ = session.tx.send(crate::mcp::Message::Notification(n_clone.clone())).await;
            }
        });
        
        // Also send it to the main rx for consistency
        server.notification_tx.send(notification).await.unwrap();
        let _received_by_main = rx.recv().await.unwrap();

        // 3. Receive it via SSE
        let second_event = body.next().await.unwrap().unwrap();
        let event_str = String::from_utf8(second_event.to_vec()).unwrap();
        
        assert!(event_str.contains("event: message"));
        assert!(event_str.contains("test/notification"));
        assert!(event_str.contains("bar"));
    }
}
