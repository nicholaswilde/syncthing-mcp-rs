use crate::api::SyncThingClient;
use crate::config::{AppConfig, BandwidthLimits, InstanceConfig};
use crate::tools::bandwidth::BandwidthController;
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_set_instance_bandwidth_limits() {
        let server = MockServer::start().await;

        // Mock getting the config
        let initial_config = json!({
            "version": 37,
            "folders": [],
            "devices": [],
            "gui": {},
            "ldap": {},
            "options": {
                "maxRecvKbps": 0,
                "maxSendKbps": 0
            },
            "remoteIgnoredDevices": [],
            "defaults": {},
            "pendingDevices": {}
        });

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(initial_config.clone()))
            .expect(1)
            .mount(&server)
            .await;

        // Mock setting the config
        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let config = InstanceConfig {
            url: server.uri(),
            api_key: Some("test-key".to_string()),
            name: Some("test-instance".to_string()),
            ..Default::default()
        };
        let client = SyncThingClient::new(config);

        let limits = BandwidthLimits {
            max_recv_kbps: Some(1000),
            max_send_kbps: Some(500),
        };

        let controller = BandwidthController::new();
        let result = controller
            .set_instance_bandwidth_limits(&client, limits)
            .await;

        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_bandwidth_limits_global() {
        let server1 = MockServer::start().await;
        let server2 = MockServer::start().await;

        let initial_config = json!({
            "version": 37,
            "folders": [],
            "devices": [],
            "gui": {},
            "ldap": {},
            "options": {
                "maxRecvKbps": 0,
                "maxSendKbps": 0
            },
            "remoteIgnoredDevices": [],
            "defaults": {},
            "pendingDevices": {}
        });

        // Setup both servers
        for server in [&server1, &server2] {
            Mock::given(method("GET"))
                .and(path("/rest/config"))
                .respond_with(ResponseTemplate::new(200).set_body_json(initial_config.clone()))
                .expect(1)
                .mount(server)
                .await;

            Mock::given(method("PUT"))
                .and(path("/rest/config"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(server)
                .await;
        }

        let app_config = AppConfig {
            instances: vec![
                InstanceConfig {
                    url: server1.uri(),
                    api_key: Some("key1".to_string()),
                    name: Some("instance1".to_string()),
                    ..Default::default()
                },
                InstanceConfig {
                    url: server2.uri(),
                    api_key: Some("key2".to_string()),
                    name: Some("instance2".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let limits = BandwidthLimits {
            max_recv_kbps: Some(2000),
            max_send_kbps: Some(1000),
        };

        let controller = BandwidthController::new();
        // Update all instances
        let result = controller
            .update_bandwidth_limits(&app_config, None, limits)
            .await;

        assert!(result.is_ok());
    }
}
