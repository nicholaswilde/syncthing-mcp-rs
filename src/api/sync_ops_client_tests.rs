#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::InstanceConfig;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use serde_json::json;

    #[tokio::test]
    async fn test_set_file_priority() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("POST"))
            .and(path("/rest/db/prio"))
            .and(query_param("folder", "default"))
            .and(query_param("file", "test.txt"))
            .and(header("X-API-Key", api_key))
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
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.set_file_priority("default", "test.txt").await.unwrap();

        assert_eq!(result.total, Some(0));
    }

    #[tokio::test]
    async fn test_get_device_completion_with_folder() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/db/completion"))
            .and(query_param("device", "device1"))
            .and(query_param("folder", "folder1"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "completion": 50.0,
                "globalBytes": 1000,
                "needBytes": 500,
                "globalItems": 10,
                "needItems": 5,
                "needDeletes": 0,
                "remoteState": "valid",
                "sequence": 100
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.get_device_completion("device1", Some("folder1")).await.unwrap();

        assert_eq!(result.completion, 50.0);
        assert_eq!(result.global_bytes, 1000);
    }
}
