#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::devices::inspect_device;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_inspect_device_success() {
        let server = MockServer::start().await;

        // Mock system config
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": 37,
                "folders": [],
                "devices": [
                    {
                        "deviceID": "device1",
                        "name": "Device 1",
                        "addresses": ["dynamic"],
                        "compression": "metadata",
                        "introducer": false,
                        "paused": false,
                        "untrusted": false
                    }
                ],
                "gui": {},
                "ldap": {},
                "options": {},
                "remoteIgnoredDevices": [],
                "defaults": {}
            })))
            .mount(&server)
            .await;

        // Mock device completion
        Mock::given(method("GET"))
            .and(path("/rest/db/completion"))
            .and(query_param("device", "device1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "completion": 95.5,
                "globalBytes": 10000,
                "needBytes": 450
            })))
            .mount(&server)
            .await;

        // Mock device stats
        Mock::given(method("GET"))
            .and(path("/rest/stats/device"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "device1": {
                    "lastSeen": "2023-01-01T12:00:00Z",
                    "lastConnectionDurationS": 3600.0
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
        let args = json!({
            "device_id": "device1"
        });

        let result = inspect_device(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("Device Overview: Device 1 (device1)"));
        assert!(text.contains("**Completion**: 95.50%"));
        assert!(text.contains("**Last Seen**: 2023-01-01T12:00:00Z"));
    }

    #[tokio::test]
    async fn test_inspect_device_json_optimized() {
        let server = MockServer::start().await;

        // Mock system config
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": 37,
                "folders": [],
                "devices": [
                    {
                        "deviceID": "device1",
                        "name": "Device 1",
                        "addresses": ["dynamic"]
                    }
                ],
                "gui": {},
                "ldap": {},
                "options": {},
                "remoteIgnoredDevices": [],
                "defaults": {}
            })))
            .mount(&server)
            .await;

        // Mock device completion
        Mock::given(method("GET"))
            .and(path("/rest/db/completion"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "completion": 95.5,
                "globalBytes": 10000,
                "needBytes": 450
            })))
            .mount(&server)
            .await;

        // Mock device stats
        Mock::given(method("GET"))
            .and(path("/rest/stats/device"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "device1": {
                    "lastSeen": "2023-01-01T12:00:00Z",
                    "lastConnectionDurationS": 3600.0
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
        let args = json!({
            "device_id": "device1",
            "format": "json",
            "shorten": true
        });

        let result = inspect_device(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        let json: serde_json::Value = serde_json::from_str(text).unwrap();
        println!("JSON: {}", serde_json::to_string_pretty(&json).unwrap());

        // Check for aliased fields
        // completion -> cp (both as field name and sub-field)
        assert!(json["cp"]["cp"].is_number());
        // lastSeen -> ls
        assert!(json["stats"]["ls"].is_string());
    }
}


