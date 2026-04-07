#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::config::patch_instance_config;
    use serde_json::json;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_patch_instance_config_folder() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";
        let folder_id = "folder1";
        let patch = json!({
            "label": "Updated Label"
        });

        // Mock PATCH
        Mock::given(method("PATCH"))
            .and(path(format!("/rest/config/folders/{}", folder_id)))
            .and(header("X-API-Key", api_key))
            .and(body_json(patch.clone()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": folder_id,
                "label": "Updated Label",
                "path": "/tmp/folder1",
                "type": "sendreceive"
            })))
            .mount(&mock_server)
            .await;

        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("default".to_string()),
                url: mock_server.uri(),
                api_key: Some(api_key.to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let client = SyncThingClient::new(config.instances[0].clone());
        let args = json!({
            "folder_id": folder_id,
            "patch": patch
        });

        let result = patch_instance_config(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("Successfully patched folder: folder1"));
        assert!(text.contains("Updated Label"));
    }

    #[tokio::test]
    async fn test_patch_instance_config_dry_run() {
        let mock_server = MockServer::start().await;
        let folder_id = "folder1";
        let patch = json!({
            "label": "New Label"
        });

        // Mock GET for current state
        Mock::given(method("GET"))
            .and(path(format!("/rest/config/folders/{}", folder_id)))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": folder_id,
                "label": "Old Label",
                "path": "/tmp/folder1",
                "type": "sendreceive",
                "devices": []
            })))
            .mount(&mock_server)
            .await;

        let config = AppConfig {
            instances: vec![InstanceConfig {
                name: Some("default".to_string()),
                url: mock_server.uri(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let client = SyncThingClient::new(config.instances[0].clone());
        let args = json!({
            "folder_id": folder_id,
            "patch": patch,
            "dry_run": true
        });

        let result = patch_instance_config(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("Dry Run: Proposed changes"));
        assert!(text.contains("\"label\""));
        assert!(text.contains("\"New Label\""));
    }
}
