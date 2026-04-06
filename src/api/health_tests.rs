#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::InstanceConfig;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_health_check_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/system/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "version": "v1.0.0",
                "arch": "amd64",
                "os": "linux",
                "isRelease": true,
                "isBeta": false,
                "isCandidate": false
            })))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
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
            .and(path("/rest/system/config/insync"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "insync": true
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some("test-api-key".to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let health = client.health_check().await.unwrap();

        assert_eq!(health.status, "Online");
        assert!(health.version.is_some());
        assert_eq!(health.version.unwrap(), "v1.0.0");
        assert_eq!(health.uptime.unwrap(), 100);
        assert_eq!(health.memory_alloc.unwrap(), 1000);
        assert_eq!(health.memory_sys.unwrap(), 2000);
        assert_eq!(health.config_insync.unwrap(), true);
        assert!(health.error.is_none());
    }

    #[tokio::test]
    async fn test_health_check_failure() {
        let mock_server = MockServer::start().await;

        // Mock a failure
        Mock::given(method("GET"))
            .and(path("/rest/system/version"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some("test-api-key".to_string()),
            retry_max_attempts: Some(1),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let health = client.health_check().await.unwrap();

        assert_eq!(health.status, "Offline");
        assert!(health.version.is_none());
        assert!(health.error.is_some());
    }
}
