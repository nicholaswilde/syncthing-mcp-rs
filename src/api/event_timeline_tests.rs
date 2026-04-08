#[cfg(test)]
mod tests {
    use crate::api::client::SyncThingClient;
    use crate::config::InstanceConfig;
    use chrono::{Utc, Duration as ChronoDuration};
    use std::time::Duration;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_events_since_duration() {
        let mock_server = MockServer::start().await;
        let now = Utc::now();
        
        let e1_time = (now - ChronoDuration::minutes(10)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let e2_time = (now - ChronoDuration::minutes(4)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let e3_time = (now - ChronoDuration::minutes(1)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": 1,
                    "type": "Starting",
                    "time": e1_time,
                    "data": null
                },
                {
                    "id": 2,
                    "type": "DeviceConnected",
                    "time": e2_time,
                    "data": {
                        "device": "d1",
                        "addr": "1.2.3.4",
                        "type": "tcp-client"
                    }
                },
                {
                    "id": 3,
                    "type": "FolderStateChanged",
                    "time": e3_time,
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

        // Fetch events from the last 5 minutes
        let events = client.get_events_since_duration(Duration::from_secs(300)).await.unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, 2);
        assert_eq!(events[1].id, 3);
    }

    #[tokio::test]
    async fn test_get_events_since_time() {
        let mock_server = MockServer::start().await;
        let now = Utc::now();
        let cutoff = now - ChronoDuration::minutes(5);
        
        let e1_time = (now - ChronoDuration::minutes(10)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let e2_time = (now - ChronoDuration::minutes(4)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        Mock::given(method("GET"))
            .and(path("/rest/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": 1,
                    "type": "Starting",
                    "time": e1_time,
                    "data": null
                },
                {
                    "id": 2,
                    "type": "DeviceConnected",
                    "time": e2_time,
                    "data": null
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

        let events = client.get_events_since_time(cutoff).await.unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, 2);
    }
}
