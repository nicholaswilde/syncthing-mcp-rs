use crate::api::client::SyncThingClient;
use crate::api::models::GuiConfig;
use crate::config::{AppConfig, InstanceConfig};
use crate::tools::gui_settings::{get_gui_settings, update_gui_settings};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_gui_settings() {
    let mock_server = MockServer::start().await;
    let api_key = "test-api-key";

    Mock::given(method("GET"))
        .and(path("/rest/config/gui"))
        .and(header("X-API-Key", api_key))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
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

    let config = AppConfig {
        instances: vec![InstanceConfig {
            name: Some("test-instance".to_string()),
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let client = SyncThingClient::new(config.instances[0].clone());
    let args = json!({});

    let result = get_gui_settings(client, config, args).await.unwrap();

    assert_eq!(result["enabled"], true);
    assert_eq!(result["address"], "127.0.0.1:8384");
    assert_eq!(result["user"], "********");
    assert_eq!(result["password"], "********");
    assert_eq!(result["apiKey"], "********");
    assert_eq!(result["useTLS"], true);
}

#[tokio::test]
async fn test_update_gui_settings() {
    let mock_server = MockServer::start().await;
    let api_key = "test-api-key";

    Mock::given(method("GET"))
        .and(path("/rest/config/gui"))
        .and(header("X-API-Key", api_key))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
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

    Mock::given(method("PUT"))
        .and(path("/rest/config/gui"))
        .and(header("X-API-Key", api_key))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let config = AppConfig {
        instances: vec![InstanceConfig {
            name: Some("test-instance".to_string()),
            url: mock_server.uri(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let client = SyncThingClient::new(config.instances[0].clone());
    let args = json!({
        "enabled": false,
        "theme": "light",
        "user": "new_admin"
    });

    let result = update_gui_settings(client, config, args).await.unwrap();

    assert_eq!(result["message"], "GUI settings updated successfully");
}
