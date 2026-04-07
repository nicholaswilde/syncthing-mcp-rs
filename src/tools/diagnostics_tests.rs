#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::AppConfig;
    use crate::config::InstanceConfig;
    use crate::tools::diagnostics::*;
    use serde_json::json;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_file_info_tool() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/db/file"))
            .and(query_param("folder", "default"))
            .and(query_param("file", "test.txt"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "availability": [],
                "global": {
                    "name": "test.txt",
                    "type": "FILE_INFO_TYPE_FILE",
                    "size": 100,
                    "permissions": 420,
                    "modifiedS": 123456789,
                    "modifiedNs": 0,
                    "modifiedBy": "device1",
                    "version": {"counters": []},
                    "sequence": 1,
                    "noPermissions": false,
                    "invalid": false,
                    "deleted": false,
                    "ignored": false,
                    "mustRescan": false
                },
                "local": {
                    "name": "test.txt",
                    "type": "FILE_INFO_TYPE_FILE",
                    "size": 100,
                    "permissions": 420,
                    "modifiedS": 123456789,
                    "modifiedNs": 0,
                    "modifiedBy": "device1",
                    "version": {"counters": []},
                    "sequence": 1,
                    "noPermissions": false,
                    "invalid": false,
                    "deleted": false,
                    "ignored": false,
                    "mustRescan": false
                },
                "mtime": {
                    "err": null,
                    "value": {
                        "real": "2023-01-01T00:00:00Z",
                        "virtual": "2023-01-01T00:00:00Z"
                    }
                }
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        let client = SyncThingClient::new(config.clone());
        let app_config = AppConfig {
            instances: vec![config],
            ..Default::default()
        };

        let params = json!({
            "folder_id": "default",
            "file_path": "test.txt"
        });

        let result = get_file_info(client, app_config, params).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("File Info for 'test.txt'"));
    }

    #[tokio::test]
    async fn test_get_folder_needs_tool() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/db/need"))
            .and(query_param("folder", "default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "progress": [],
                "queued": [],
                "rest": [
                    {
                        "type": "FILE_INFO_TYPE_FILE",
                        "sequence": 1,
                        "modified": "2023-01-01T00:00:00Z",
                        "name": "need.txt",
                        "size": 100,
                        "version": []
                    }
                ],
                "page": 1,
                "perpage": 100,
                "total": 1
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        let client = SyncThingClient::new(config.clone());
        let app_config = AppConfig {
            instances: vec![config],
            ..Default::default()
        };

        let params = json!({
            "folder_id": "default"
        });

        let result = get_folder_needs(client, app_config, params).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Folder Needs: default"));
        assert!(text.contains("need.txt"));
    }

    #[tokio::test]
    async fn test_get_discovery_status_tool() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/system/discovery"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "device1": {
                    "addresses": ["tcp://1.2.3.4:22000"]
                }
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        let client = SyncThingClient::new(config.clone());
        let app_config = AppConfig {
            instances: vec![config],
            ..Default::default()
        };

        let params = json!({});

        let result = get_discovery_status(client, app_config, params)
            .await
            .unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Discovery Status (1 devices)"));
        assert!(text.contains("device1"));
    }

    #[tokio::test]
    async fn test_diagnose_network_issues_tool() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        // Mock discovery
        Mock::given(method("GET"))
            .and(path("/rest/system/discovery"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "DEVICE-1": {
                    "addresses": ["tcp://192.168.1.100:22000"]
                },
                "DEVICE-2": {
                    "addresses": ["relay://1.2.3.4:22067"]
                }
            })))
            .mount(&mock_server)
            .await;

        // Mock connections
        Mock::given(method("GET"))
            .and(path("/rest/system/connections"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": {
                    "inBytesTotal": 0,
                    "outBytesTotal": 0
                },
                "connections": {
                    "DEVICE-1": {
                        "connected": false,
                        "paused": false,
                    },
                    "DEVICE-2": {
                        "connected": true,
                        "type": "relay-client",
                        "paused": false,
                    }
                }
            })))
            .mount(&mock_server)
            .await;

        // Mock system status to get "my ID" so we don't flag ourselves
        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "myID": "MY-DEVICE-ID",
                "uptime": 1234,
                "alloc": 1024,
                "sys": 2048,
                "goroutines": 10,
                "pathSeparator": "/"
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        let client = SyncThingClient::new(config.clone());
        let app_config = AppConfig {
            instances: vec![config],
            ..Default::default()
        };

        let params = json!({});

        let result = diagnose_network_issues(client, app_config, params)
            .await
            .unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Network Diagnostics Report"));
        assert!(text.contains("DEVICE: DEVICE-1"));
        assert!(text.contains("Status: Offline"));
        assert!(text.contains("DEVICE: DEVICE-2"));
        assert!(text.contains("Status: Connected (Degraded via Relay)"));
    }
}
