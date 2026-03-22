#[cfg(test)]
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
        let result = client.patch_device("test-device-id", serde_json::json!({"paused": true})).await;

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
        let result = client.patch_folder("default", serde_json::json!({"paused": true})).await;

        assert!(result.is_ok());
    }
}
