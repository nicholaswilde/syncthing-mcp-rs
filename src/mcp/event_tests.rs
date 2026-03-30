#[cfg(test)]
mod tests {
    use crate::config::{AppConfig, InstanceConfig};
    use crate::mcp::Notification;
    use crate::mcp::events::EventManager;
    use tokio::sync::mpsc;
    use tokio::time::{Duration, timeout};
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
            mcp_events: vec!["FolderStateChanged".to_string()],
            ..Default::default()
        };

        let (tx, mut rx) = mpsc::channel::<Notification>(100);
        let event_manager = EventManager::new(config, tx);

        // Run one iteration manually or use a short sleep
        let em_clone = event_manager.clone();
        let handle = tokio::spawn(async move {
            let _ = em_clone.run().await;
        });

        // Wait for notification with timeout
        let notification = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for notification")
            .expect("Channel closed");

        assert_eq!(notification.method, "notifications/message");
        assert_eq!(notification.params.as_ref().unwrap()["instance"], "test");

        event_manager.stop();
        let _ = timeout(Duration::from_secs(1), handle).await;
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
        let handle = tokio::spawn(async move {
            let _ = em_clone.run().await;
        });

        // Use a timeout to wait for no notification
        let result = tokio::time::timeout(tokio::time::Duration::from_millis(200), rx.recv()).await;
        assert!(
            result.is_err(),
            "Expected no notification for filtered event"
        );

        event_manager.stop();
        let _ = timeout(Duration::from_secs(1), handle).await;
    }

    #[tokio::test]
    async fn test_mcp_server_receives_events_from_manager() {
        use crate::mcp::server::McpServer;
        use crate::tools::create_registry;
        use tokio::io::AsyncReadExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

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
            mcp_events: vec!["FolderStateChanged".to_string()],
            ..Default::default()
        };

        let registry = create_registry();
        let (server, rx) = McpServer::new(registry, config.clone());
        let event_manager = EventManager::new(config, server.notification_tx.clone());

        // Spawn manager
        let em_clone = event_manager.clone();
        let em_handle = tokio::spawn(async move {
            let _ = em_clone.run().await;
        });

        // Run server in memory
        let (client_writer, server_reader) = tokio::io::duplex(1024);
        let (server_writer, mut client_reader) = tokio::io::duplex(1024);

        let server_clone = server.clone();
        let server_handle = tokio::spawn(async move {
            server_clone
                .run(server_reader, server_writer, rx)
                .await
                .unwrap();
        });

        // Read notification from client side
        let mut buffer = [0u8; 1024];
        let n = timeout(Duration::from_secs(2), client_reader.read(&mut buffer))
            .await
            .expect("Timeout reading notification")
            .unwrap();

        // Use a Deserializer to handle potentially multiple messages (trailing characters error fix)
        let mut de = serde_json::Deserializer::from_slice(&buffer[..n]);
        let msg: crate::mcp::Message = serde::Deserialize::deserialize(&mut de).unwrap();

        if let crate::mcp::Message::Notification(notification) = msg {
            assert_eq!(notification.method, "notifications/message");
            let params = notification.params.unwrap();
            assert_eq!(params["instance"], "test");
            assert!(
                params["summary"]
                    .as_str()
                    .unwrap()
                    .contains("Folder 'f1' changed state")
            );
        } else {
            panic!("Expected notification");
        }

        // Cleanup
        event_manager.stop();
        let _ = timeout(Duration::from_secs(1), em_handle).await;
        server.stop();
        drop(client_writer);
        let _ = timeout(Duration::from_secs(1), server_handle).await;
    }

    #[tokio::test]
    async fn test_event_manager_multi_instance() {
        let mock_server1 = MockServer::start().await;
        let mock_server2 = MockServer::start().await;

        let event_resp1 = serde_json::json!([{"id": 1, "type": "DeviceConnected", "time": "2023", "data": {"device": "d1", "addr": "a1", "type": "t1"}}]);
        let event_resp2 = serde_json::json!([{"id": 1, "type": "DeviceConnected", "time": "2023", "data": {"device": "d2", "addr": "a2", "type": "t2"}}]);

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&event_resp1))
            .mount(&mock_server1)
            .await;
        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&event_resp2))
            .mount(&mock_server2)
            .await;

        let config = AppConfig {
            instances: vec![
                InstanceConfig {
                    name: Some("inst1".to_string()),
                    url: mock_server1.uri(),
                    ..Default::default()
                },
                InstanceConfig {
                    name: Some("inst2".to_string()),
                    url: mock_server2.uri(),
                    ..Default::default()
                },
            ],
            mcp_events: vec!["DeviceConnected".to_string()],
            ..Default::default()
        };

        let (tx, mut rx) = mpsc::channel(100);
        let event_manager = EventManager::new(config, tx);

        let em_clone = event_manager.clone();
        let handle = tokio::spawn(async move {
            let _ = em_clone.run().await;
        });

        let n1 = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for n1")
            .unwrap();
        let n2 = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for n2")
            .unwrap();

        let instances = [
            n1.params.as_ref().unwrap()["instance"]
                .as_str()
                .unwrap()
                .to_string(),
            n2.params.as_ref().unwrap()["instance"]
                .as_str()
                .unwrap()
                .to_string(),
        ];

        assert!(instances.contains(&"inst1".to_string()));
        assert!(instances.contains(&"inst2".to_string()));

        event_manager.stop();
        let _ = timeout(Duration::from_secs(1), handle).await;
    }

    #[tokio::test(start_paused = true)]
    async fn test_event_manager_sequence() {
        let mock_server = MockServer::start().await;

        let event_resp1 = serde_json::json!([
            {
                "id": 1,
                "type": "FolderStateChanged",
                "time": "2023-01-01T00:00:00Z",
                "data": {"folder": "f1", "from": "idle", "to": "syncing"}
            }
        ]);

        let event_resp2 = serde_json::json!([
            {
                "id": 2,
                "type": "FolderStateChanged",
                "time": "2023-01-01T00:00:05Z",
                "data": {"folder": "f1", "from": "syncing", "to": "idle"}
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .and(wiremock::matchers::query_param_is_missing("since"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&event_resp1))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .and(query_param("since", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&event_resp2))
            .mount(&mock_server)
            .await;

        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("test".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            mcp_events: vec!["FolderStateChanged".to_string()],
            ..Default::default()
        };

        let (tx, mut rx) = mpsc::channel::<Notification>(100);
        let event_manager = EventManager::new(config, tx);

        // We need to run it in a loop but we want to control it
        let em_clone = event_manager.clone();
        let handle = tokio::spawn(async move {
            let _ = em_clone.run().await;
        });

        // Yield to allow the spawned task to run
        tokio::task::yield_now().await;

        // First notification
        let n1 = rx.recv().await.expect("Failed to receive n1");
        assert!(
            n1.params.unwrap()["summary"]
                .as_str()
                .unwrap()
                .contains("idle to syncing")
        );

        // Advance time to trigger next poll (interval is 5s)
        tokio::time::advance(std::time::Duration::from_secs(6)).await;
        // Yield again to allow the loop to wake up and process
        tokio::task::yield_now().await;

        // Second notification
        let n2 = rx.recv().await.expect("Failed to receive n2");
        assert!(
            n2.params.unwrap()["summary"]
                .as_str()
                .unwrap()
                .contains("syncing to idle")
        );

        event_manager.stop();
        // Since clock is paused, we might need to advance it once more to let the select! finish
        tokio::time::advance(std::time::Duration::from_millis(10)).await;
        let _ = handle.await;
    }
}
