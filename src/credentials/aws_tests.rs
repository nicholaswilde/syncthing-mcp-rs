use super::*;

#[tokio::test]
async fn test_aws_backend_creation() {
    // Just verify we can create it without crashing
    let _backend = AwsBackend::new("us-east-1".to_string(), None).await;
}
