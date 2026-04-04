#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::conflicts::summarize_conflicts;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_summarize_conflicts_success() {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir().unwrap();

        let folder1_path = temp.path().join("folder1");
        tokio::fs::create_dir(&folder1_path).await.unwrap();
        let folder2_path = temp.path().join("folder2");
        tokio::fs::create_dir(&folder2_path).await.unwrap();

        // Create a conflict in folder1
        let conflict1_name = "test1.sync-conflict-20230101-120000-ABCDEFG.txt";
        tokio::fs::write(folder1_path.join(conflict1_name), "conflict1")
            .await
            .unwrap();

        // Mock list_folders
        Mock::given(method("GET"))
            .and(path("/rest/config/folders"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "folder1",
                    "path": folder1_path.to_string_lossy(),
                    "label": "Folder 1",
                    "type": "sendreceive",
                    "devices": []
                },
                {
                    "id": "folder2",
                    "path": folder2_path.to_string_lossy(),
                    "label": "Folder 2",
                    "type": "sendreceive",
                    "devices": []
                }
            ])))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({});

        let result = summarize_conflicts(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("SyncThing Conflicts Summary"));
        assert!(text.contains("Folder: Folder 1 (folder1)"));
        assert!(text.contains("1 conflict(s) found"));
        assert!(text.contains("Folder: Folder 2 (folder2)"));
        assert!(text.contains("No conflicts found"));
    }

    #[tokio::test]
    async fn test_summarize_conflicts_json_optimized() {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir().unwrap();

        let folder1_path = temp.path().join("folder1");
        tokio::fs::create_dir(&folder1_path).await.unwrap();

        // Create a conflict in folder1
        let conflict1_name = "test1.sync-conflict-20230101-120000-ABCDEFG.txt";
        tokio::fs::write(folder1_path.join(conflict1_name), "conflict1")
            .await
            .unwrap();

        // Mock list_folders
        Mock::given(method("GET"))
            .and(path("/rest/config/folders"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": "folder1",
                    "path": folder1_path.to_string_lossy(),
                    "label": "Folder 1",
                    "type": "sendreceive",
                    "devices": []
                }
            ])))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({
            "format": "json",
            "limit": 1
        });

        let result = summarize_conflicts(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        let json: serde_json::Value = serde_json::from_str(text).unwrap();

        assert!(json["folders"].is_array());
        assert_eq!(json["folders"].as_array().unwrap().len(), 1);
        assert!(json["folders"][0]["conflicts"].is_array());
        assert_eq!(json["folders"][0]["conflicts"].as_array().unwrap().len(), 1);
    }
}
