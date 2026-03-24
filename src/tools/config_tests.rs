#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::AppConfig;
    use crate::tools::config::replicate_config;
    use serde_json::json;
    use wiremock::http::Method;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_replicate_config_dry_run() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        // Source config
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": 37,
                "folders": [{"id": "folder1"}],
                "devices": [{"deviceID": "device1"}]
            })))
            .mount(&source_mock)
            .await;

        // Destination config (get)
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": 37,
                "folders": [],
                "devices": []
            })))
            .mount(&dest_mock)
            .await;

        // Destination config (PUT) - should NOT be called
        let put_mock = Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200));
        dest_mock.register(put_mock).await;

        let config = AppConfig {
            instances: vec![
                crate::config::InstanceConfig {
                    name: Some("source".to_string()),
                    url: source_mock.uri(),
                    ..Default::default()
                },
                crate::config::InstanceConfig {
                    name: Some("dest".to_string()),
                    url: dest_mock.uri(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let client = SyncThingClient::new(config.instances[0].clone());
        let args = json!({
            "source": "source",
            "destination": "dest",
            "dry_run": true
        });

        let result = replicate_config(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("[DRY RUN]"));
        assert!(text.contains("Would replicate configuration to dest"));
        assert!(text.contains("Folders: 1 added, 0 removed, 0 updated."));

        // Verify PUT was NOT called
        let received_requests = dest_mock.received_requests().await.unwrap();
        let put_requests: Vec<_> = received_requests
            .into_iter()
            .filter(|r| r.method == Method::PUT && r.url.path() == "/rest/config")
            .collect();
        assert_eq!(put_requests.len(), 0, "PUT /rest/config should not have been called in dry-run mode");
    }

    #[tokio::test]
    async fn test_replicate_config_invalid_filters() {
        let mock_server = MockServer::start().await;
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("source".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let client = SyncThingClient::new(config.instances[0].clone());

        // 1. folders not an array
        let args = json!({
            "destination": "source",
            "folders": "not-an-array"
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(_))));

        // 2. devices not an array
        let args = json!({
            "destination": "source",
            "devices": 123
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_replicate_config_filter_not_found() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [{"id": "exists"}],
                "devices": [{"deviceID": "exists"}]
            })))
            .mount(&source_mock)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&dest_mock)
            .await;

        let config = AppConfig {
            instances: vec![
                crate::config::InstanceConfig {
                    name: Some("source".to_string()),
                    url: source_mock.uri(),
                    ..Default::default()
                },
                crate::config::InstanceConfig {
                    name: Some("dest".to_string()),
                    url: dest_mock.uri(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let client = SyncThingClient::new(config.instances[0].clone());

        // Folder not found
        let args = json!({
            "source": "source",
            "destination": "dest",
            "folders": ["non-existent"]
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(result.is_err());
        if let Err(crate::error::Error::ValidationError(msg)) = result {
            assert!(msg.contains("Folder not found in source: non-existent"));
        } else {
            panic!("Expected ValidationError, got {:?}", result);
        }

        // Device not found
        let args = json!({
            "source": "source",
            "destination": "dest",
            "devices": ["non-existent"]
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(result.is_err());
        if let Err(crate::error::Error::ValidationError(msg)) = result {
            assert!(msg.contains("Device not found in source: non-existent"));
        } else {
            panic!("Expected ValidationError, got {:?}", result);
        }

        // Folder ID not string
        let args = json!({
            "source": "source",
            "destination": "dest",
            "folders": [123]
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(msg)) if msg.contains("folder IDs must be strings")));

        // Device ID not string
        let args = json!({
            "source": "source",
            "destination": "dest",
            "devices": [true]
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(msg)) if msg.contains("device IDs must be strings")));
    }

    #[tokio::test]
    async fn test_replicate_config_tool_schema() {
        use crate::tools::create_registry;
        let registry = create_registry();
        let tool = registry.get_tool("replicate_config").unwrap();
        let schema = tool.input_schema;

        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("dry_run"));
        assert!(props.contains_key("folders"));
        assert!(props.contains_key("devices"));

        assert_eq!(props["dry_run"]["type"], "boolean");
        assert_eq!(props["folders"]["type"], "array");
        assert_eq!(props["devices"]["type"], "array");
    }

    #[tokio::test]
    async fn test_config_diff_generation() {
        use crate::tools::config_diff::ConfigDiff;

        let source = json!({
            "folders": [
                {"id": "folder1", "label": "Folder 1"},
                {"id": "folder2", "label": "Folder 2"}
            ],
            "devices": [
                {"deviceID": "device1", "name": "Device 1"},
                {"deviceID": "device2", "name": "Device 2"}
            ]
        });

        let dest = json!({
            "folders": [
                {"id": "folder1", "label": "Folder 1"},
                {"id": "folder3", "label": "Folder 3"}
            ],
            "devices": [
                {"deviceID": "device1", "name": "Device 1"},
                {"deviceID": "device3", "name": "Device 3"}
            ]
        });

        let diff = ConfigDiff::generate(&source, &dest);

        assert_eq!(diff.folders_added, vec!["folder2"]);
        assert_eq!(diff.folders_removed, vec!["folder3"]);
        assert_eq!(diff.folders_updated, vec!["folder1"]);
        assert_eq!(diff.devices_added, vec!["device2"]);
        assert_eq!(diff.devices_removed, vec!["device3"]);
        assert_eq!(diff.devices_updated, vec!["device1"]);

        let summary = diff.summary();
        assert!(summary.contains("Folders: 1 added, 1 removed, 1 updated."));
        assert!(summary.contains("Devices: 1 added, 1 removed, 1 updated."));
        assert!(summary.contains("+ Folder: folder2"));
        assert!(summary.contains("- Folder: folder3"));
        assert!(summary.contains("~ Folder: folder1"));
    }
}
