#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::folders::{
        configure_sharing, get_folder_stats, manage_folders, manage_ignores,
    };
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_manage_folders_list() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/config/folders"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
                "id": "folder1",
                "label": "Folder 1",
                "path": "/tmp",
                "type": "sendreceive",
                "paused": false,
                "devices": []
            }])))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({"action": "list"});

        let result = manage_folders(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Folders:"));
        assert!(text.contains("Folder 1 (folder1)"));
    }

    #[tokio::test]
    async fn test_manage_folders_stats() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/stats/folder"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folder1": {
                    "lastScan": "2023-01-01T12:00:00Z",
                    "lastFile": {
                        "filename": "test.txt",
                        "at": "2023-01-01T12:00:00Z"
                    }
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
        let args = json!({"action": "stats"});

        let result = get_folder_stats(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Folder Statistics:"));
        assert!(text.contains("folder1"));
        assert!(text.contains("test.txt"));
    }

    #[tokio::test]
    async fn test_manage_folders_share() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/config/folders/folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "folder1",
                "label": "Folder 1",
                "path": "/tmp",
                "type": "sendreceive",
                "devices": []
            })))
            .mount(&server)
            .await;

        Mock::given(method("PATCH"))
            .and(path("/rest/config/folders/folder1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({
            "action": "share",
            "folder_id": "folder1",
            "device_id": "device1"
        });

        let result = configure_sharing(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("shared with device device1 successfully"));
    }

    #[tokio::test]
    async fn test_manage_folders_ignores_get() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/db/ignores"))
            .and(query_param("folder", "folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "ignore": ["node_modules", ".git"]
            })))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({"action": "get", "folder_id": "folder1"});

        let result = manage_ignores(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Ignore patterns for folder folder1:"));
        assert!(text.contains("node_modules"));
    }
}
