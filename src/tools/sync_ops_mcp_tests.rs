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
}
