#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::InstanceConfig;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_system_status() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .and(header("X-API-Key", api_key))
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

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let status = client.get_system_status().await.unwrap();

        assert_eq!(status.my_id, "test-id");
        assert_eq!(status.uptime, 100);
    }

    #[tokio::test]
    async fn test_get_system_version() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/system/version"))
            .and(header("X-API-Key", api_key))
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

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let version = client.get_system_version().await.unwrap();

        assert_eq!(version.version, "v1.0.0");
        assert!(version.is_release);
    }

    #[tokio::test]
    async fn test_get_config() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "version": 37,
                "folders": [],
                "devices": []
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let config_data = client.get_config().await.unwrap();

        assert_eq!(config_data["version"], 37);
    }

    #[tokio::test]
    async fn test_set_config() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let new_config = serde_json::json!({
            "version": 37,
            "folders": [],
            "devices": []
        });

        let result = client.set_config(new_config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_folders() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/config/folders"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": "default",
                    "label": "Default Folder",
                    "path": "/home/sync",
                    "type": "sendreceive",
                    "devices": [
                        {"deviceID": "device1"}
                    ],
                    "rescan_interval_s": 3600,
                    "fs_watcher_enabled": true,
                    "paused": false
                }
            ])))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let folders = client.list_folders().await.unwrap();

        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].id, "default");
        assert_eq!(folders[0].label, "Default Folder");
    }

    #[tokio::test]
    async fn test_add_folder() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("POST"))
            .and(path("/rest/config/folders"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.add_folder("new-folder", "New Folder", "/path").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_folder() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/config/folders/default"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "default",
                "label": "Default Folder",
                "path": "/home/sync",
                "type": "sendreceive",
                "devices": [],
                "rescan_interval_s": 3600,
                "fs_watcher_enabled": true,
                "paused": false
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let folder = client.get_folder("default").await.unwrap();

        assert_eq!(folder.id, "default");
    }

    #[tokio::test]
    async fn test_patch_folder() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("PATCH"))
            .and(path("/rest/config/folders/default"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client
            .patch_folder("default", serde_json::json!({"paused": true}))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_devices() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/config/devices"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "deviceID": "test-device-id",
                    "name": "Test Device",
                    "addresses": ["dynamic"],
                    "compression": "metadata",
                    "introducer": false,
                    "paused": false,
                    "untrusted": false
                }
            ])))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let devices = client.list_devices().await.unwrap();

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, "test-device-id");
        assert_eq!(devices[0].name.as_ref().unwrap(), "Test Device");
    }

    #[tokio::test]
    async fn test_add_device() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("POST"))
            .and(path("/rest/config/devices"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.add_device("new-device-id", Some("New Device")).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_device() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("DELETE"))
            .and(path("/rest/config/devices/test-device-id"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.remove_device("test-device-id").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_patch_device() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("PATCH"))
            .and(path("/rest/config/devices/test-device-id"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client
            .patch_device("test-device-id", serde_json::json!({"paused": true}))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_ignores() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/db/ignores"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ignore": ["node_modules", "*.tmp"],
                "expanded": ["node_modules", "*.tmp"]
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let ignores = client.get_ignores("default").await.unwrap();

        assert_eq!(ignores.ignore.as_ref().unwrap().len(), 2);
        assert_eq!(ignores.ignore.as_ref().unwrap()[0], "node_modules");
    }

    #[tokio::test]
    async fn test_set_ignores() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("POST"))
            .and(path("/rest/db/ignores"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client
            .set_ignores("default", vec!["new_pattern".to_string()])
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_folder_status() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/db/status"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "state": "idle",
                "needBytes": 0,
                "needFiles": 0,
                "inSyncBytes": 1000,
                "inSyncFiles": 10,
                "globalBytes": 1000,
                "globalFiles": 10,
                "localBytes": 1000,
                "localFiles": 10
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let status = client.get_folder_status("default").await.unwrap();

        assert_eq!(status.state, "idle");
        assert_eq!(status.in_sync_bytes, 1000);
    }

    #[tokio::test]
    async fn test_get_device_completion() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/db/completion"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "completion": 100.0,
                "needBytes": 0,
                "needFiles": 0,
                "globalBytes": 1000
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let completion = client.get_device_completion("test-device").await.unwrap();

        assert_eq!(completion.completion, 100.0);
        assert_eq!(completion.global_bytes, 1000);
    }

    #[tokio::test]
    async fn test_rescan() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("POST"))
            .and(path("/rest/db/scan"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        client.rescan(Some("test-folder")).await.unwrap();
    }

    #[tokio::test]
    async fn test_revert_folder() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("POST"))
            .and(path("/rest/db/revert"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        client.revert_folder("test-folder").await.unwrap();
    }

    #[tokio::test]
    async fn test_restart() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("POST"))
            .and(path("/rest/system/restart"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        client.restart().await.unwrap();
    }

    #[tokio::test]
    async fn test_clear_errors() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("POST"))
            .and(path("/rest/system/error/clear"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        client.clear_errors().await.unwrap();
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        // First request fails with 500, second succeeds
        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
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

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let status = client.get_system_status().await.unwrap();

        assert_eq!(status.my_id, "test-id");
    }

    #[tokio::test]
    async fn test_retry_failure() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        // Always fails with 500
        Mock::given(method("GET"))
            .and(path("/rest/system/status"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            retry_max_attempts: Some(2),
            retry_initial_backoff_ms: Some(1), // Fast retry for tests
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.get_system_status().await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::Error::Api(_)));
    }

    #[tokio::test]
    async fn test_get_events() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": 1,
                    "type": "Starting",
                    "time": "2023-01-01T00:00:00Z",
                    "data": null
                },
                {
                    "id": 2,
                    "type": "FolderSummary",
                    "time": "2023-01-01T00:00:01Z",
                    "data": {"folder": "default"}
                }
            ])))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let events = client.get_events(None, None).await.unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, 1);
        assert_eq!(events[0].event_type, "Starting");
        assert_eq!(events[1].id, 2);
        assert_eq!(events[1].event_type, "FolderSummary");
    }

    #[tokio::test]
    async fn test_browse() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/db/browse"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "subdir": {},
                "file.txt": [123456789, 1024]
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.browse("default", None, Some(1)).await.unwrap();

        assert!(result.get("subdir").is_some());
        assert!(result.get("file.txt").is_some());
    }

    #[tokio::test]
    async fn test_get_pending_devices() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/cluster/pending/devices"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "DEVICE-ID": {
                    "time": "2023-01-01T00:00:00Z",
                    "name": "test-device",
                    "address": "1.2.3.4:22000"
                }
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let pending = client.get_pending_devices().await.unwrap();

        assert_eq!(pending.len(), 1);
        assert_eq!(pending["DEVICE-ID"].name, "test-device");
    }

    #[tokio::test]
    async fn test_remove_pending_device() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("DELETE"))
            .and(path("/rest/cluster/pending/devices"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.remove_pending_device("DEVICE-ID").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_connections() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/system/connections"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "DEVICE-ID-1": {
                    "at": "2023-10-24T12:34:56Z",
                    "inBytesTotal": 1000,
                    "outBytesTotal": 2000,
                    "address": "1.2.3.4:22000",
                    "clientVersion": "v1.27.0",
                    "connected": true,
                    "type": "tcp-client",
                    "isPaused": false
                }
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let connections = client.get_connections().await.unwrap();

        assert_eq!(connections.len(), 1);
        let conn = &connections["DEVICE-ID-1"];
        assert!(conn.connected);
        assert_eq!(conn.in_bytes_total, 1000);
        assert_eq!(conn.client_version, Some("v1.27.0".to_string()));
    }

    #[tokio::test]
    async fn test_get_system_log() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/system/log"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "messages": [
                    {
                        "when": "2023-10-27T10:00:00Z",
                        "message": "test log message"
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let log = client.get_system_log().await.unwrap();

        assert_eq!(log.messages.len(), 1);
        assert_eq!(log.messages[0].message, "test log message");
    }

    #[tokio::test]
    async fn test_get_device_stats() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/stats/device"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "DEVICE-ID-1": {
                    "lastSeen": "2023-10-27T15:33:10Z",
                    "lastConnectionDurationS": 3600.5
                }
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let stats = client.get_device_stats().await.unwrap();

        assert_eq!(stats.len(), 1);
        assert_eq!(stats["DEVICE-ID-1"].last_connection_duration_s, 3600.5);
    }

    #[tokio::test]
    async fn test_get_folder_stats() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/stats/folder"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "folder1": {
                    "lastScan": "2023-10-27T14:20:01Z",
                    "lastFile": {
                        "filename": "test.txt",
                        "at": "2023-10-27T14:19:55Z"
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

        let client = SyncThingClient::new(config);
        let stats = client.get_folder_stats().await.unwrap();

        assert_eq!(stats.len(), 1);
        assert_eq!(stats["folder1"].last_file.as_ref().unwrap().filename, "test.txt");
    }

    #[tokio::test]
    async fn test_get_pending_folders() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/cluster/pending/folders"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "folder1": {
                    "offeredBy": {
                        "DEVICE1": {
                            "time": "2023-10-27T10:00:00Z",
                            "label": "Test Folder",
                            "receiveEncrypted": false,
                            "remoteEncrypted": false
                        }
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

        let client = SyncThingClient::new(config);
        let pending = client.get_pending_folders().await.unwrap();

        assert_eq!(pending.len(), 1);
        assert_eq!(pending["folder1"].offered_by["DEVICE1"].label, "Test Folder");
    }

    #[tokio::test]
    async fn test_remove_pending_folder() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("DELETE"))
            .and(path("/rest/cluster/pending/folders"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.remove_pending_folder("folder1").await;

        assert!(result.is_ok());
    }
}
