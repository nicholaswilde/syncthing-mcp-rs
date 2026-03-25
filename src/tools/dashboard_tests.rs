#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::api::models::*;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::dashboard::get_global_dashboard;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn mock_system_status() -> SystemStatus {
        SystemStatus {
            my_id: "test-id".to_string(),
            uptime: 3600,
            alloc: 1024 * 1024,
            total_memory: 2048 * 1024,
            goroutines: 10,
            path_separator: "/".to_string(),
        }
    }

    fn mock_system_version() -> SystemVersion {
        SystemVersion {
            version: "v1.23.4".to_string(),
            arch: "amd64".to_string(),
            os: "linux".to_string(),
            is_release: true,
            is_beta: false,
            is_candidate: false,
        }
    }

    fn mock_connections() -> ConnectionsResponse {
        let mut connections = std::collections::HashMap::new();
        connections.insert(
            "device1".to_string(),
            ConnectionStatus {
                connected: true,
                in_bytes_total: 1024,
                out_bytes_total: 512,
                ..Default::default()
            },
        );

        ConnectionsResponse { connections }
    }

    #[tokio::test]
    async fn test_get_global_dashboard_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/system/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_system_version()))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_system_status()))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/system/connections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_connections()))
            .mount(&mock_server)
            .await;

        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("Instance 1".to_string()),
                url: mock_server.uri(),
                api_key: Some("test-key".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let client = SyncThingClient::new(config.instances[0].clone());
        let result = get_global_dashboard(client, config, json!({}))
            .await
            .unwrap();

        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Global SyncThing Dashboard"));
        assert!(text.contains("1/1 instances online"));
        assert!(text.contains("Instance 1"));
        assert!(text.contains("↓ 1.00 KB"));
        assert!(text.contains("↑ 512 B"));
    }

    #[tokio::test]
    async fn test_get_global_dashboard_offline_instance() {
        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("Offline Instance".to_string()),
                url: "http://localhost:12345".to_string(), // Invalid port
                retry_max_attempts: Some(1),
                timeout_s: Some(1),
                ..Default::default()
            }],
            ..Default::default()
        };

        let client = SyncThingClient::new(config.instances[0].clone());
        let result = get_global_dashboard(client, config, json!({}))
            .await
            .unwrap();

        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("0/1 instances online"));
        assert!(text.contains("🔴 **Offline Instance**"));
    }
}
