#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::folders::{
        configure_sharing, get_folder_stats, manage_folders, manage_ignores,
    };
    use serde_json::{Value, json};
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
        assert_eq!(result["content"][0]["type"], "json");
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
                        "DEVICE1": {
                            "time": "2023-10-27T10:00:00Z",
                            "label": "Test Folder",
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
        assert!(text.contains("DEVICE1"));
    }

    #[tokio::test]
    async fn test_manage_folders_pending_empty() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/cluster/pending/folders"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
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
        assert!(text.contains("No pending folder requests."));
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
        assert!(text.contains("triggered revert for folder: folder1"));
    }

    #[tokio::test]
    async fn test_configure_sharing_unshare() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/config/folders/folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "folder1",
                "label": "Folder 1",
                "path": "/tmp",
                "type": "sendreceive",
                "devices": [{"deviceID": "device1"}]
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
            "action": "unshare",
            "folder_id": "folder1",
            "device_id": "device1"
        });

        let result = configure_sharing(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("unshared from device device1 successfully"));
    }

    #[tokio::test]
    async fn test_manage_ignores_set() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/db/ignores"))
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
            "action": "set",
            "folder_id": "folder1",
            "patterns": ["pattern1", "pattern2"]
        });

        let result = manage_ignores(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully set 2 ignore patterns"));
    }

    #[tokio::test]
    async fn test_manage_ignores_append() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/db/ignores"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "ignore": ["pattern1"]
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/rest/db/ignores"))
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
            "action": "append",
            "folder_id": "folder1",
            "patterns": ["pattern2"]
        });

        let result = manage_ignores(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully appended 1 new ignore patterns"));
    }

    #[tokio::test]
    async fn test_inspect_folder() {
        use crate::tools::folders::inspect_folder;
        let server = MockServer::start().await;

        // Mock folder config
        Mock::given(method("GET"))
            .and(path("/rest/config/folders/folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "folder1",
                "label": "Folder 1",
                "path": "/tmp",
                "type": "sendreceive",
                "devices": [{"deviceID": "device1"}]
            })))
            .mount(&server)
            .await;

        // Mock folder status
        Mock::given(method("GET"))
            .and(path("/rest/db/status"))
            .and(query_param("folder", "folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "state": "idle",
                "globalBytes": 1000,
                "inSyncBytes": 1000,
                "globalFiles": 10,
                "inSyncFiles": 10,
                "localBytes": 1000,
                "localFiles": 10,
                "needBytes": 0,
                "needFiles": 0
            })))
            .mount(&server)
            .await;

        // Mock folder stats
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

        // Mock device completion
        Mock::given(method("GET"))
            .and(path("/rest/db/completion"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "completion": 100.0,
                "globalBytes": 1000,
                "needBytes": 0,
                "globalItems": 10,
                "needItems": 0,
                "needDeletes": 0,
                "remoteState": "valid",
                "sequence": 1
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
            "folder_id": "folder1",
            "include_devices": true
        });

        let result = inspect_folder(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Folder Overview: Folder 1 (folder1)"));
        assert!(text.contains("**Completion**: 100.00%"));
        assert!(text.contains("device1"));
    }

    #[tokio::test]
    async fn test_inspect_folder_json() {
        use crate::tools::folders::inspect_folder;
        let server = MockServer::start().await;

        // Mock folder config
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

        // Mock folder status
        Mock::given(method("GET"))
            .and(path("/rest/db/status"))
            .and(query_param("folder", "folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "state": "idle",
                "globalBytes": 1000,
                "inSyncBytes": 1000,
                "globalFiles": 10,
                "inSyncFiles": 10,
                "localBytes": 1000,
                "localFiles": 10,
                "needBytes": 0,
                "needFiles": 0
            })))
            .mount(&server)
            .await;

        // Mock folder stats
        Mock::given(method("GET"))
            .and(path("/rest/stats/folder"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folder1": { "lastScan": "2023-01-01T12:00:00Z" }
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
            "folder_id": "folder1",
            "format": "json",
            "shorten": false
        });

        let result = inspect_folder(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        let json: Value = serde_json::from_str(text).unwrap();
        assert_eq!(json["folder_id"], "folder1");
        assert_eq!(json["status"]["state"], "idle");
    }

    #[tokio::test]
    async fn test_batch_manage_folders() {
        use crate::tools::folders::batch_manage_folders;
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/rest/db/scan"))
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
            "folder_ids": ["folder1", "folder2"],
            "action": "rescan"
        });

        let result = batch_manage_folders(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully triggered rescan for 2 folder(s):"));
        assert!(text.contains("folder1: Success"));
        assert!(text.contains("folder2: Success"));
    }

    #[tokio::test]
    async fn test_set_file_priority() {
        use crate::tools::folders::set_file_priority;
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/rest/db/prio"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "progress": [],
                "queued": [],
                "rest": [],
                "page": 1,
                "perpage": 10
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
            "folder_id": "folder1",
            "file_path": "test.txt"
        });

        let result = set_file_priority(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Priority set successfully for 'test.txt'"));
    }
}
