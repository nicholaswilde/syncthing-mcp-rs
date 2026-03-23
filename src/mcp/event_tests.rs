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
}
