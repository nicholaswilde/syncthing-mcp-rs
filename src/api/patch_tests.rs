#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::InstanceConfig;
    use wiremock::matchers::{header, method, path, body_json};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use serde_json::json;

    #[tokio::test]
    async fn test_patch_folder_config() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";
        let folder_id = "test-folder";
        let patch = json!({
            "label": "Updated Label",
            "paused": true
        });

        Mock::given(method("PATCH"))
            .and(path(format!("/rest/config/folders/{}", folder_id)))
            .and(header("X-API-Key", api_key))
            .and(body_json(patch.clone()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": folder_id,
                "label": "Updated Label",
                "paused": true
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let result = client.patch_folder_config(folder_id, patch).await.unwrap();

        assert_eq!(result["label"], "Updated Label");
        assert_eq!(result["paused"], true);
    }
}
