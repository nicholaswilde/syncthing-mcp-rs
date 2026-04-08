#[cfg(test)]
mod tests {
    use crate::config::{AppConfig, HttpServerConfig};
    use crate::mcp::server::McpServer;
    use crate::mcp::{Message, Request, RequestId};
    use crate::tools::create_registry;
    use axum::{
        body::Body,
        http::{Request as HttpRequest, StatusCode},
    };
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn test_auth_middleware_no_key() {
        let registry = create_registry();
        let config = AppConfig {
            http_server: HttpServerConfig {
                enabled: true,
                api_key: None,
                ..Default::default()
            },
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);
        let app = server.router();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/message?session_id=123")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&Message::Request(Request {
                            jsonrpc: "2.0".to_string(),
                            id: RequestId::Number(1),
                            method: "initialize".to_string(),
                            params: None,
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should be 404 because session doesn't exist, but NOT 401
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_auth_middleware_with_key_unauthorized() {
        let registry = create_registry();
        let config = AppConfig {
            http_server: HttpServerConfig {
                enabled: true,
                api_key: Some("secret-key".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);
        let app = server.router();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/message?session_id=123")
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_with_key_authorized() {
        let registry = create_registry();
        let config = AppConfig {
            http_server: HttpServerConfig {
                enabled: true,
                api_key: Some("secret-key".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let (server, _rx) = McpServer::new(registry, config);
        let app = server.router();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/message?session_id=123")
                    .header("Authorization", "Bearer secret-key")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&Message::Request(Request {
                            jsonrpc: "2.0".to_string(),
                            id: RequestId::Number(1),
                            method: "initialize".to_string(),
                            params: None,
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should be 404 because session doesn't exist, but it PASSED the auth middleware
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_message_handler_success() {
        let registry = create_registry();
        let config = AppConfig::default();
        let (server, _rx) = McpServer::new(registry, config);

        // Add a mock session
        let session_id = "test-session".to_string();
        let (tx, _rx_session) = tokio::sync::mpsc::channel(10);
        server
            .sessions
            .insert(session_id.clone(), crate::mcp::server::Session { tx });

        let app = server.router();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri(format!("/message?session_id={}", session_id))
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&Message::Request(Request {
                            jsonrpc: "2.0".to_string(),
                            id: RequestId::Number(1),
                            method: "initialize".to_string(),
                            params: None,
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_sse_handler() {
        let registry = create_registry();
        let config = AppConfig::default();
        let (server, _rx) = McpServer::new(registry, config);
        let app = server.router();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .method("GET")
                    .uri("/sse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }
}
