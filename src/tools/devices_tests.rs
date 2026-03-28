#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::devices::manage_devices;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_manage_devices_discover() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/cluster/pending/devices"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "device1": {
                    "address": "1.2.3.4:22000",
                    "name": "Pending Device",
                    "time": "2023-10-27T10:00:00Z"
                }
            })))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let params = json!({
            "action": "discover"
        });

        let result = manage_devices(client, config, params).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Pending Device Requests:"));
        assert!(text.contains("Pending Device"));
        assert!(text.contains("device1"));
    }

    #[tokio::test]
    async fn test_manage_devices_discover_empty() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/cluster/pending/devices"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let params = json!({
            "action": "discover"
        });

        let result = manage_devices(client, config, params).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("No pending device requests."));
    }

    #[tokio::test]
    async fn test_manage_devices_approve() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/rest/config/devices"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        Mock::given(method("DELETE"))
            .and(path("/rest/cluster/pending/devices/device1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let params = json!({
            "action": "approve",
            "device_id": "device1",
            "name": "Approved Device"
        });

        let result = manage_devices(client, config, params).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Device device1 approved and added successfully"));
    }

    #[tokio::test]
    async fn test_manage_devices_invalid_action() {
        let client = SyncThingClient::new(InstanceConfig {
            url: "http://localhost".to_string(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let params = json!({
            "action": "invalid"
        });

        let result = manage_devices(client, config, params).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported action")
        );
    }
}
