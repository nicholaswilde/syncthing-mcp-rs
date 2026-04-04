#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::browser::browse_folder;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_browse_folder_optimized() {
        let server = MockServer::start().await;

        // Mock browse
        Mock::given(method("GET"))
            .and(path("/rest/db/browse"))
            .and(query_param("folder", "folder1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "name": "file1.txt",
                    "size": 1024,
                    "modTime": "2023-01-01T12:00:00Z"
                },
                {
                    "name": "file2.txt",
                    "size": 2048,
                    "modTime": "2023-01-01T12:00:00Z"
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
            "folder_id": "folder1",
            "limit": 1
        });

        let result = browse_folder(client, config, args).await.unwrap();
        // Since browse_folder currently returns the raw JSON from client.browse
        let json = result;

        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 1);
        assert_eq!(json[0]["name"], "file1.txt");
    }
}
