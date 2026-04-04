#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::folders::batch_manage_folders;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_batch_manage_folders_rescan() {
        let server = MockServer::start().await;

        // Mock rescan for folder1
        Mock::given(method("POST"))
            .and(path("/rest/db/scan"))
            .and(query_param("folder", "folder1"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        // Mock rescan for folder2
        Mock::given(method("POST"))
            .and(path("/rest/db/scan"))
            .and(query_param("folder", "folder2"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
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

        assert!(text.contains("Successfully triggered rescan for 2 folder(s)"));
        assert!(text.contains("folder1: Success"));
        assert!(text.contains("folder2: Success"));
    }

    #[tokio::test]
    async fn test_batch_manage_folders_revert() {
        let server = MockServer::start().await;

        // Mock revert for folder1
        Mock::given(method("POST"))
            .and(path("/rest/db/revert"))
            .and(query_param("folder", "folder1"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({
            "folder_ids": ["folder1"],
            "action": "revert"
        });

        let result = batch_manage_folders(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("Successfully triggered revert for 1 folder(s)"));
        assert!(text.contains("folder1: Success"));
    }
}
