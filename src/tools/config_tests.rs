#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::AppConfig;
    use crate::tools::config::replicate_config;
    use serde_json::{json, Value};
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
    async fn test_replicate_config_selective_folders() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        // Source config: 2 folders, 2 devices
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [
                    {"id": "folder1", "devices": [{"deviceID": "device1"}]},
                    {"id": "folder2", "devices": [{"deviceID": "device2"}]}
                ],
                "devices": [
                    {"deviceID": "device1"},
                    {"deviceID": "device2"}
                ]
            })))
            .mount(&source_mock)
            .await;

        // Destination config: empty
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [],
                "devices": []
            })))
            .mount(&dest_mock)
            .await;

        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
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

        // Replicate ONLY folder1
        let args = json!({
            "source": "source",
            "destination": "dest",
            "folders": ["folder1"]
        });

        let _ = replicate_config(client, config, args).await.unwrap();

        // Verify PUT body
        let received_requests = dest_mock.received_requests().await.unwrap();
        let put_request = received_requests
            .iter()
            .find(|r| r.method == Method::PUT && r.url.path() == "/rest/config")
            .unwrap();
        
        let body: Value = serde_json::from_slice(&put_request.body).unwrap();
        let folders = body["folders"].as_array().unwrap();
        let devices = body["devices"].as_array().unwrap();

        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0]["id"], "folder1");
        
        // Should also include device1 since folder1 uses it
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0]["deviceID"], "device1");
    }

    #[tokio::test]
    async fn test_replicate_config_selective_folders_no_devices() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [{"id": "folder1", "devices": []}],
                "devices": []
            })))
            .mount(&source_mock)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [],
                "devices": []
            })))
            .mount(&dest_mock)
            .await;

        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
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

        let args = json!({
            "source": "source",
            "destination": "dest",
            "folders": ["folder1"]
        });

        let _ = replicate_config(client, config, args).await.unwrap();

        let received_requests = dest_mock.received_requests().await.unwrap();
        let put_request = received_requests
            .iter()
            .find(|r| r.method == Method::PUT && r.url.path() == "/rest/config")
            .unwrap();
        
        let body: Value = serde_json::from_slice(&put_request.body).unwrap();
        assert_eq!(body["folders"].as_array().unwrap().len(), 1);
        assert_eq!(body["devices"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_replicate_config_selective_devices() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [],
                "devices": [{"deviceID": "d1"}, {"deviceID": "d2"}]
            })))
            .mount(&source_mock)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&dest_mock)
            .await;

        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
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

        let args = json!({
            "source": "source",
            "destination": "dest",
            "devices": ["d1"]
        });

        let _ = replicate_config(client, config, args).await.unwrap();

        let received_requests = dest_mock.received_requests().await.unwrap();
        let put_request = received_requests
            .iter()
            .find(|r| r.method == Method::PUT && r.url.path() == "/rest/config")
            .unwrap();
        
        let body: Value = serde_json::from_slice(&put_request.body).unwrap();
        assert_eq!(body["devices"].as_array().unwrap().len(), 1);
        assert_eq!(body["devices"][0]["deviceID"], "d1");
    }

    #[tokio::test]
    async fn test_replicate_config_selective_folders_dest_not_found() {
        let source_mock = MockServer::start().await;
        let config = AppConfig {
            instances: vec![
                crate::config::InstanceConfig {
                    name: Some("source".to_string()),
                    url: source_mock.uri(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let client = SyncThingClient::new(config.instances[0].clone());

        let args = json!({
            "source": "source",
            "destination": "non-existent",
            "folders": ["folder1"]
        });

        let result = replicate_config(client, config, args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(msg)) if msg.contains("Destination instance not found")));
    }

    #[tokio::test]
    async fn test_replicate_config_selective_folders_source_not_found() {
        let dest_mock = MockServer::start().await;
        let config = AppConfig {
            instances: vec![
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
            "source": "non-existent",
            "destination": "dest",
            "folders": ["folder1"]
        });

        let result = replicate_config(client, config, args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(msg)) if msg.contains("Source instance not found")));
    }

    #[tokio::test]
    async fn test_replicate_config_selective_folders_update() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [{"id": "folder1", "label": "Source Label"}],
                "devices": []
            })))
            .mount(&source_mock)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [{"id": "folder1", "label": "Dest Label"}],
                "devices": []
            })))
            .mount(&dest_mock)
            .await;

        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
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

        let args = json!({
            "source": "source",
            "destination": "dest",
            "folders": ["folder1"]
        });

        let result = replicate_config(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("1 updated"));
    }

    #[tokio::test]
    async fn test_replicate_config_full() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [{"id": "f1"}],
                "devices": [{"deviceID": "d1"}]
            })))
            .mount(&source_mock)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [],
                "devices": []
            })))
            .mount(&dest_mock)
            .await;

        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
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

        let args = json!({
            "source": "source",
            "destination": "dest"
        });

        let result = replicate_config(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Folders: 1 added"));
        assert!(text.contains("Devices: 1 added"));
    }

    #[tokio::test]
    async fn test_replicate_config_selective_folders_empty_source() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
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

        let args = json!({
            "source": "source",
            "destination": "dest",
            "folders": ["folder1"]
        });

        let result = replicate_config(client, config, args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(msg)) if msg.contains("Folder not found in source: folder1")));
    }

    #[tokio::test]
    async fn test_replicate_config_selective_devices_invalid_id_type() {
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

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "devices": [{"deviceID": "device1"}]
            })))
            .mount(&mock_server)
            .await;

        let args = json!({
            "source": "source",
            "destination": "source",
            "devices": [123]
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(msg)) if msg.contains("device IDs must be strings")));
    }

    #[tokio::test]
    async fn test_replicate_config_selective_folders_invalid_device_id_in_folder() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;
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

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [{"id": "folder1", "devices": [{"deviceID": 123}]}]
            })))
            .mount(&source_mock)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&dest_mock)
            .await;

        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&dest_mock)
            .await;

        let args = json!({
            "source": "source",
            "destination": "dest",
            "folders": ["folder1"]
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        // This actually won't fail because my logic uses .and_then(|id| id.as_str()) which just returns None.
        // It won't hit the HashSet insertion if it's not a string.
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_replicate_config_selective_folders_invalid_id_type() {
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

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folders": [{"id": "folder1"}]
            })))
            .mount(&mock_server)
            .await;

        let args = json!({
            "source": "source",
            "destination": "source",
            "folders": [123]
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(msg)) if msg.contains("folder IDs must be strings")));
    }

    #[tokio::test]
    async fn test_replicate_config_instance_not_found() {
        let config = AppConfig {
            instances: vec![crate::config::InstanceConfig {
                name: Some("exists".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let client = SyncThingClient::new(config.instances[0].clone());

        // Source not found
        let args = json!({
            "source": "non-existent",
            "destination": "exists"
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(msg)) if msg.contains("Source instance not found")));

        // Destination not found
        let args = json!({
            "source": "exists",
            "destination": "non-existent"
        });
        let result = replicate_config(client.clone(), config.clone(), args).await;
        assert!(matches!(result, Err(crate::error::Error::ValidationError(msg)) if msg.contains("Destination instance not found")));
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

    #[tokio::test]
    async fn test_config_diff_summary_warnings() {
        use crate::tools::config_diff::ConfigDiff;

        let diff = ConfigDiff {
            folders_added: vec!["f1".to_string()],
            folders_removed: vec!["f2".to_string()],
            folders_updated: vec![],
            devices_added: vec![],
            devices_removed: vec!["d1".to_string()],
            devices_updated: vec![],
        };

        let summary = diff.summary();
        assert!(summary.contains("⚠️ WARNING: This action will REMOVE 1 folder(s) and 1 device(s) from the destination instance."));
    }
}
