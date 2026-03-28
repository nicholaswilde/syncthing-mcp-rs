#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::browser::browse_folder;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_browse_folder_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/db/browse"))
            .and(query_param("folder", "folder1"))
            .and(query_param("levels", "1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!([{"name": "file1.txt", "size": 123}])),
            )
            .mount(&server)
            .await;

        let client = SyncThingClient::new(InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let params = json!({
            "folder_id": "folder1",
            "levels": 1
        });

        let result = browse_folder(client, config, params).await.unwrap();
        assert_eq!(result, json!([{"name": "file1.txt", "size": 123}]));
    }

    #[tokio::test]
    async fn test_browse_folder_missing_id() {
        let client = SyncThingClient::new(InstanceConfig {
            url: "http://localhost".to_string(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let params = json!({});

        let result = browse_folder(client, config, params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("folder_id is required"));
    }
}
