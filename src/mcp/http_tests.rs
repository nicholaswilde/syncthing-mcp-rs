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
}
