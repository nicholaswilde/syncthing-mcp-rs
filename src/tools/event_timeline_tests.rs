#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::{AppConfig, InstanceConfig};
    use crate::tools::event_timeline::get_event_timeline;
    use chrono::{Utc, Duration as ChronoDuration};
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_event_timeline_tool() {
        let mock_server = MockServer::start().await;
        let now = Utc::now();
        let e1_time = (now - ChronoDuration::minutes(10)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": 1,
                    "type": "FolderStateChanged",
                    "time": e1_time,
                    "data": {
                        "folder": "f1",
                        "from": "idle",
                        "to": "syncing"
                    }
                }
            ])))
            .mount(&mock_server)
            .await;

        let config = InstanceConfig {
            url: mock_server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        };
        let client = SyncThingClient::new(config);
        let app_config = AppConfig::default();

        let args = json!({ "duration_s": 3600 });
        let result = get_event_timeline(client, app_config, args).await.unwrap();

        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Folder 'f1' changed state from idle to syncing"));
    }
}
