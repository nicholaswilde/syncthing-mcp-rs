#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::AppConfig;
    use crate::tools::config::replicate_config;
    use serde_json::json;
    use wiremock::http::Method;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_replicate_config_dry_run() {
        let source_mock = MockServer::start().await;
        let dest_mock = MockServer::start().await;

        // Source config
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": 37,
                "folders": [{"id": "folder1"}],
                "devices": [{"deviceID": "device1"}]
            })))
            .mount(&source_mock)
            .await;

        // Destination config (get)
        Mock::given(method("GET"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": 37,
                "folders": [],
                "devices": []
            })))
            .mount(&dest_mock)
            .await;

        // Destination config (PUT) - should NOT be called
        let put_mock = Mock::given(method("PUT"))
            .and(path("/rest/config"))
            .respond_with(ResponseTemplate::new(200));
        dest_mock.register(put_mock).await;

        let config = AppConfig {
            instances: vec![
                crate::config::InstanceConfig {
                    name: Some("source".to_string()),
                    url: source_mock.uri(),
                    ..Default::default()
                },
                crate::config::InstanceConfig {
                    name: Some("dest".to_string()),
                    url: dest_mock.uri(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let client = SyncThingClient::new(config.instances[0].clone());
        let args = json!({
            "source": "source",
            "destination": "dest",
            "dry_run": true
        });

        let result = replicate_config(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("[DRY RUN]"));
        assert!(text.contains("Would replicate configuration to dest"));
        assert!(text.contains("Folders: 1 added, 0 removed, 0 updated."));

        // Verify PUT was NOT called
        let received_requests = dest_mock.received_requests().await.unwrap();
        let put_requests: Vec<_> = received_requests
            .into_iter()
            .filter(|r| r.method == Method::PUT && r.url.path() == "/rest/config")
            .collect();
        assert_eq!(put_requests.len(), 0, "PUT /rest/config should not have been called in dry-run mode");
    }
}
