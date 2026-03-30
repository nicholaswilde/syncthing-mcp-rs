use super::*;

#[tokio::test]
async fn test_aws_backend_creation() {
    // Just verify we can create it without crashing
    let _backend = AwsBackend::new("us-east-1".to_string(), None).await;
}

#[tokio::test]
async fn test_aws_config_registration() {
    let mut config = crate::config::AppConfig {
        aws: crate::config::AwsConfig {
            enabled: true,
            region: "us-east-1".to_string(),
            profile: None,
        },
        ..Default::default()
    };
    
    // This should register the backend
    config.validate().await.unwrap();
    
    // We can't easily verify the registry content here without making BACKEND_REGISTRY public
    // or adding a get_backend function. But we can check if resolve_api_key tries to use it.
}
