#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::api::models::GuiConfig;
    use crate::config::InstanceConfig;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_gui_config() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("GET"))
            .and(path("/rest/config/gui"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "enabled": true,
                "address": "127.0.0.1:8384",
                "user": "admin",
                "password": "hashed_password",
                "useTLS": true,
                "apiKey": "some-api-key",
                "theme": "dark",
                "debugging": false,
                "insecureAdminAccess": false,
                "insecureSkipHostcheck": false,
                "insecureAllowFrameAuth": false
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);
        let gui_config = client.get_gui_config().await.unwrap();

        assert!(gui_config.enabled);
        assert_eq!(gui_config.address, "127.0.0.1:8384");
        assert_eq!(gui_config.user.as_deref(), Some("admin"));
        assert!(gui_config.use_tls);
    }

    #[tokio::test]
    async fn test_set_gui_config() {
        let mock_server = MockServer::start().await;
        let api_key = "test-api-key";

        Mock::given(method("PUT")) // Or PATCH, depending on Syncthing API, PUT is standard for /rest/config/gui
            .and(path("/rest/config/gui"))
            .and(header("X-API-Key", api_key))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };

        let client = SyncThingClient::new(config);

        let mut new_gui_config = GuiConfig::default();
        new_gui_config.enabled = true;
        new_gui_config.theme = "light".to_string();

        let result = client.set_gui_config(&new_gui_config).await;

        assert!(result.is_ok());
    }
}
