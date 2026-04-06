#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::AppConfig;
    use crate::config::InstanceConfig;
    use crate::tools::system::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_check_upgrade_tool() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/system/upgrade"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "latest": "v1.2.0",
                "newer": true,
                "majorNewer": false,
                "running": "v1.1.0"
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            ..Default::default()
        };
        let client = SyncThingClient::new(config.clone());
        let app_config = AppConfig {
            instances: vec![config],
            ..Default::default()
        };

        let result = check_upgrade(client, app_config, json!({})).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("SyncThing Upgrade Check"));
        assert!(text.contains("v1.2.0"));
    }

    #[tokio::test]
    async fn test_perform_upgrade_tool() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/system/upgrade"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            ..Default::default()
        };
        let client = SyncThingClient::new(config.clone());
        let app_config = AppConfig {
            instances: vec![config],
            ..Default::default()
        };

        let result = perform_upgrade(client, app_config, json!({}))
            .await
            .unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Successfully triggered SyncThing upgrade"));
    }

    #[tokio::test]
    async fn test_ping_instance_tool() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/system/ping"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "ping": "pong"
            })))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            ..Default::default()
        };
        let client = SyncThingClient::new(config.clone());
        let app_config = AppConfig {
            instances: vec![config],
            ..Default::default()
        };

        let result = ping_instance(client, app_config, json!({})).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Ping response: pong"));
    }
}
