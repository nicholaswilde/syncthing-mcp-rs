#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::system::{get_system_errors, is_config_insync};
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_is_config_insync_tool() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/system/config/insync"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "configInSync": true
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            ..Default::default()
        };
        let client = SyncThingClient::new(config);
        let app_config = AppConfig::default();
        let args = json!({});

        let result = is_config_insync(client, app_config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Configuration is in sync"));
    }

    #[tokio::test]
    async fn test_get_system_errors_tool() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/system/error"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "errors": [
                    {
                        "when": "2023-10-27T10:00:00Z",
                        "message": "test error message"
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            ..Default::default()
        };
        let client = SyncThingClient::new(config);
        let app_config = AppConfig::default();
        let args = json!({});

        let result = get_system_errors(client, app_config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("test error message"));
    }
}
