#[cfg(test)]
mod tests {
    use crate::config::{AppConfig, InstanceConfig};
    use crate::mcp::events::EventManager;
    use crate::mcp::Notification;
    use tokio::sync::mpsc;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_event_manager_polls_and_notifies() {
        let mock_server = MockServer::start().await;
        
        let event_resp = serde_json::json!([
            {
                "id": 1,
                "type": "FolderStateChanged",
                "time": "2023-01-01T00:00:00Z",
                "data": {"folder": "default"}
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .and(query_param("limit", "10"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&event_resp))
            .mount(&mock_server)
            .await;

        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("test".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let (tx, mut rx) = mpsc::channel::<Notification>(100);
        let event_manager = EventManager::new(config, tx);
        
        // Run one iteration manually or use a short sleep
        let em_clone = event_manager.clone();
        tokio::spawn(async move {
            let _ = em_clone.run().await;
        });

        let notification = rx.recv().await.unwrap();
        assert_eq!(notification.method, "notifications/message");
        assert_eq!(notification.params.unwrap()["instance"], "test");
    }

    #[tokio::test]
    async fn test_event_manager_filters_events() {
        let mock_server = MockServer::start().await;
        
        let event_resp = serde_json::json!([
            {
                "id": 1,
                "type": "ConfigSaved",
                "time": "2023-01-01T00:00:00Z",
                "data": null
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&event_resp))
            .mount(&mock_server)
            .await;

        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("test".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            // Default filter does NOT include ConfigSaved
            ..Default::default()
        };

        let (tx, mut rx) = mpsc::channel::<Notification>(100);
        let event_manager = EventManager::new(config, tx);
        
        let em_clone = event_manager.clone();
        tokio::spawn(async move {
            let _ = em_clone.run().await;
        });

        // Use a timeout to wait for no notification
        let result = tokio::time::timeout(tokio::time::Duration::from_millis(100), rx.recv()).await;
        assert!(result.is_err(), "Expected no notification for filtered event");
    }

    #[tokio::test]
    async fn test_mcp_server_receives_events_from_manager() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use crate::mcp::server::McpServer;
        use crate::tools::create_registry;
        use tokio::io::AsyncReadExt;

        let mock_server = MockServer::start().await;
        
        let event_resp = serde_json::json!([
            {
                "id": 1,
                "type": "FolderStateChanged",
                "time": "2023-01-01T00:00:00Z",
                "data": {
                    "folder": "f1",
                    "from": "idle",
                    "to": "syncing"
                }
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&event_resp))
            .mount(&mock_server)
            .await;

        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("test".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let registry = create_registry();
        let (server, rx) = McpServer::new(registry, config.clone());
        let event_manager = EventManager::new(config, server.notification_tx.clone());
        
        // Spawn manager
        let em_clone = event_manager.clone();
        tokio::spawn(async move {
            let _ = em_clone.run().await;
        });

        // Run server in memory
        let (mut client_writer, server_reader) = tokio::io::duplex(1024);
        let (server_writer, mut client_reader) = tokio::io::duplex(1024);
        
        tokio::spawn(async move {
            server.run(server_reader, server_writer, rx).await.unwrap();
        });

        // Read notification from client side
        let mut buffer = [0u8; 1024];
        let n = client_reader.read(&mut buffer).await.unwrap();
        let msg: crate::mcp::Message = serde_json::from_slice(&buffer[..n]).unwrap();
        
        if let crate::mcp::Message::Notification(notification) = msg {
            assert_eq!(notification.method, "notifications/message");
            let params = notification.params.unwrap();
            assert_eq!(params["instance"], "test");
            assert!(params["summary"].as_str().unwrap().contains("Folder 'f1' changed state"));
        } else {
            panic!("Expected notification");
        }

        // Cleanup
        drop(client_writer);
    }

    #[tokio::test]
    async fn test_event_manager_multi_instance() {
        let mock_server1 = MockServer::start().await;
        let mock_server2 = MockServer::start().await;
        
        let event_resp1 = serde_json::json!([{"id": 1, "type": "DeviceConnected", "time": "2023", "data": {"device": "d1", "addr": "a1", "type": "t1"}}]);
        let event_resp2 = serde_json::json!([{"id": 1, "type": "DeviceConnected", "time": "2023", "data": {"device": "d2", "addr": "a2", "type": "t2"}}]);

        Mock::given(method("GET")).and(path("/rest/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&event_resp1))
            .mount(&mock_server1).await;
        Mock::given(method("GET")).and(path("/rest/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&event_resp2))
            .mount(&mock_server2).await;

        let config = AppConfig {
            instances: vec![
                InstanceConfig { name: Some("inst1".to_string()), url: mock_server1.uri(), ..Default::default() },
                InstanceConfig { name: Some("inst2".to_string()), url: mock_server2.uri(), ..Default::default() },
            ],
            ..Default::default()
        };

        let (tx, mut rx) = mpsc::channel(100);
        let event_manager = EventManager::new(config, tx);
        
        let em_clone = event_manager.clone();
        tokio::spawn(async move {
            let _ = em_clone.run().await;
        });

        let n1 = rx.recv().await.unwrap();
        let n2 = rx.recv().await.unwrap();
        
        let instances = vec![
            n1.params.as_ref().unwrap()["instance"].as_str().unwrap().to_string(),
            n2.params.as_ref().unwrap()["instance"].as_str().unwrap().to_string(),
        ];
        
        assert!(instances.contains(&"inst1".to_string()));
        assert!(instances.contains(&"inst2".to_string()));
    }
}
