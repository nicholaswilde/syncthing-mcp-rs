#[cfg(test)]
mod tests {
    use crate::api::SyncThingClient;
    use crate::config::AppConfig;
    use crate::tools::system::analyze_error;
    use serde_json::json;

    #[tokio::test]
    async fn test_analyze_error_tool_unauthorized() {
        let config = AppConfig::default();
        let client = SyncThingClient::new(crate::config::InstanceConfig::default());
        let args = json!({
            "error_message": "401 Unauthorized"
        });

        let result = analyze_error(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Permission"));
        assert!(text.contains("API key"));
    }

    #[tokio::test]
    async fn test_analyze_error_tool_generic_500() {
        let config = AppConfig::default();
        let client = SyncThingClient::new(crate::config::InstanceConfig::default());
        let args = json!({
            "error_message": "folder \"abc\" not found"
        });

        let result = analyze_error(client, config, args).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Configuration"));
        assert!(text.contains("folder ID is incorrect"));
    }
}
