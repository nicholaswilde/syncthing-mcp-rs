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
    async fn test_manage_folders_get() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/config/folders/folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "folder1",
                "label": "Folder 1",
                "path": "/tmp",
                "type": "sendreceive",
                "paused": false,
                "devices": []
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

        let result = manage_folders(client, config, args).await.unwrap();
        assert_eq!(result["content"][0]["json"]["id"], "folder1");
    }

    #[tokio::test]
    async fn test_manage_folders_pending() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/cluster/pending/folders"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folder1": {
                    "offeredBy": {
                        "device1": {
                            "label": "Device 1",
                            "time": "2023-01-01T12:00:00Z",
                            "receiveEncrypted": false,
                            "remoteEncrypted": false
                        }
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
        let args = json!({"action": "pending"});

        let result = manage_folders(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Pending Folder Requests:"));
        assert!(text.contains("folder1"));
        assert!(text.contains("device1"));
    }

    #[tokio::test]
    async fn test_manage_folders_reject_pending() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/rest/cluster/pending/folders"))
            .and(query_param("folder", "folder1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({"action": "reject_pending", "folder_id": "folder1"});

        let result = manage_folders(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("rejected successfully"));
    }

    #[tokio::test]
    async fn test_manage_folders_revert() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/db/revert"))
            .and(query_param("folder", "folder1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({"action": "revert", "folder_id": "folder1"});

        let result = manage_folders(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("triggered revert"));
    }

    #[tokio::test]
    async fn test_configure_sharing_share() {
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
    async fn test_manage_ignores_get() {
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

    #[tokio::test]
    async fn test_get_folder_stats_success() {
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
        let args = json!({});

        let result = get_folder_stats(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Folder Statistics:"));
        assert!(text.contains("folder1"));
        assert!(text.contains("test.txt"));
    }
}
