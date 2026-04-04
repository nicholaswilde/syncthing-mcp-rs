#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::system::get_instance_overview;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_instance_overview_success() {
        let server = MockServer::start().await;

        // Mock system status
        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "myID": "device1",
                "uptime": 3600,
                "alloc": 512,
                "sys": 1024,
                "goroutines": 10,
                "pathSeparator": "/"
            })))
            .mount(&server)
            .await;

        // Mock system connections
        Mock::given(method("GET"))
            .and(path("/rest/system/connections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "connections": {
                    "device2": {
                        "connected": true,
                        "at": "2023-01-01T12:00:00Z"
                    }
                }
            })))
            .mount(&server)
            .await;

        // Mock system version (part of health/status check)
        Mock::given(method("GET"))
            .and(path("/rest/system/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": "v1.23.0",
                "arch": "amd64",
                "os": "linux",
                "isRelease": true,
                "isBeta": false,
                "isCandidate": false
            })))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({});

        let result = get_instance_overview(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("Instance Overview: device1"));
        assert!(text.contains("**Uptime**: 3600s"));
        assert!(text.contains("**Connected Peers**: 1 / 1"));
        assert!(text.contains("**Version**: v1.23.0"));
    }

    #[tokio::test]
    async fn test_get_instance_overview_json_optimized() {
        let server = MockServer::start().await;

        // Mock system status
        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "myID": "device1",
                "uptime": 3600,
                "alloc": 512,
                "sys": 1024,
                "goroutines": 10,
                "pathSeparator": "/"
            })))
            .mount(&server)
            .await;

        // Mock system connections
        Mock::given(method("GET"))
            .and(path("/rest/system/connections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "connections": {
                    "device2": {
                        "connected": true,
                        "at": "2023-01-01T12:00:00Z"
                    }
                }
            })))
            .mount(&server)
            .await;

        // Mock system version
        Mock::given(method("GET"))
            .and(path("/rest/system/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": "v1.23.0",
                "arch": "amd64",
                "os": "linux",
                "isRelease": true,
                "isBeta": false,
                "isCandidate": false
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
            "format": "json",
            "shorten": true
        });

        let result = get_instance_overview(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        let json: serde_json::Value = serde_json::from_str(text).unwrap();

        assert!(json["status"].is_object());
        // myID -> myID (not aliased yet, but uptime is common)
        assert!(json["status"]["uptime"].is_number());
        assert!(json["connections_summary"].is_object());
        assert_eq!(json["connections_summary"]["con"], 1);
    }
}


