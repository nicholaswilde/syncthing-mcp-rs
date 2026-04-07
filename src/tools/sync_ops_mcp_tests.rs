#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::folders::set_file_priority;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_set_file_priority_tool() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/rest/db/prio"))
            .and(query_param("folder", "default"))
            .and(query_param("file", "test.txt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "progress": [],
                "queued": [],
                "rest": [],
                "page": 1,
                "perpage": 100,
                "total": 0
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            ..Default::default()
        };
        let client = SyncThingClient::new(config);
        let app_config = AppConfig::default();
        let args = json!({
            "folder_id": "default",
            "file_path": "test.txt"
        });

        let result = set_file_priority(client, app_config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Priority set successfully"));
    }

    #[tokio::test]
    async fn test_get_device_sync_status_tool() {
        use crate::tools::devices::get_device_sync_status;
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/db/completion"))
            .and(query_param("device", "device1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "completion": 75.5,
                "globalBytes": 2000,
                "needBytes": 500,
                "globalItems": 20,
                "needItems": 5,
                "needDeletes": 0,
                "remoteState": "valid",
                "sequence": 200
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            ..Default::default()
        };
        let client = SyncThingClient::new(config);
        let app_config = AppConfig::default();
        let args = json!({
            "device_id": "device1"
        });

        let result = get_device_sync_status(client, app_config, args)
            .await
            .unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("device1"));
        assert!(text.contains("75.50%"));
    }

    #[tokio::test]
    async fn test_inspect_folder_with_devices() {
        use crate::tools::folders::inspect_folder;
        let mock_server = MockServer::start().await;

        // Mock folder config
        Mock::given(method("GET"))
            .and(path("/rest/config/folders/default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "default",
                "label": "Default",
                "path": "/tmp",
                "type": "sendreceive",
                "devices": [{"deviceID": "device1"}]
            })))
            .mount(&mock_server)
            .await;

        // Mock folder status
        Mock::given(method("GET"))
            .and(path("/rest/db/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "state": "idle",
                "globalBytes": 1000,
                "inSyncBytes": 1000
            })))
            .mount(&mock_server)
            .await;

        // Mock folder stats
        Mock::given(method("GET"))
            .and(path("/rest/stats/folder"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&mock_server)
            .await;

        // Mock device completion
        Mock::given(method("GET"))
            .and(path("/rest/db/completion"))
            .and(query_param("device", "device1"))
            .and(query_param("folder", "default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "completion": 100.0,
                "globalBytes": 1000,
                "needBytes": 0,
                "globalItems": 10,
                "needItems": 0,
                "needDeletes": 0,
                "remoteState": "valid",
                "sequence": 100
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            ..Default::default()
        };
        let client = SyncThingClient::new(config);
        let app_config = AppConfig::default();
        let args = json!({
            "folder_id": "default",
            "include_devices": true
        });

        let result = inspect_folder(client, app_config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Per-Device Completion"));
        assert!(text.contains("device1"));
    }
}
