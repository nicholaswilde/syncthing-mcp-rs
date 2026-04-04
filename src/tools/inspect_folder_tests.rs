#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::folders::inspect_folder;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_inspect_folder_success() {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir().unwrap();
        let folder_path = temp.path();

        // Mock folder config
        Mock::given(method("GET"))
            .and(path("/rest/config/folders/folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "folder1",
                "label": "Folder 1",
                "path": folder_path.to_string_lossy(),
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
                "globalBytes": 2048,
                "inSyncBytes": 1024,
                "needBytes": 1024,
                "needFiles": 1,
                "globalFiles": 2
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

        // Create a conflict file in the temp dir
        let conflict_name = "test.sync-conflict-20230101-120000-ABCDEFG.txt";
        let conflict_path = folder_path.join(conflict_name);
        tokio::fs::write(&conflict_path, "conflict content")
            .await
            .unwrap();

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({
            "folder_id": "folder1"
        });

        let result = inspect_folder(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        println!("{}", text);

        assert!(text.contains("Folder Overview: Folder 1 (folder1)"));
        assert!(text.contains("**State**: idle"));
        assert!(text.contains("**Completion**: 50.00%"));
        assert!(text.contains("#### Conflicts"));
        assert!(text.contains(conflict_name));
        assert!(text.contains("**Last Scan**: 2023-01-01T12:00:00Z"));
    }

    #[tokio::test]
    async fn test_inspect_folder_json_optimized() {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir().unwrap();
        let folder_path = temp.path();

        // Mock folder config
        Mock::given(method("GET"))
            .and(path("/rest/config/folders/folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "folder1",
                "label": "Folder 1",
                "path": folder_path.to_string_lossy(),
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
                "globalBytes": 2048,
                "inSyncBytes": 1024
            })))
            .mount(&server)
            .await;

        // Mock folder stats
        Mock::given(method("GET"))
            .and(path("/rest/stats/folder"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "folder1": {
                    "lastScan": "2023-01-01T12:00:00Z"
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
        let args = json!({
            "folder_id": "folder1",
            "format": "json",
            "shorten": true
        });

        let result = inspect_folder(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        let json: serde_json::Value = serde_json::from_str(text).unwrap();

        // Check for aliased fields
        // state -> st
        assert!(json["status"]["st"].is_string());
        assert_eq!(json["status"]["st"], "idle");
        // in_sync_bytes -> isb
        assert!(json["status"]["isb"].is_number());
        // last_scan -> ls
        assert!(json["stats"]["ls"].is_string());
    }
}
