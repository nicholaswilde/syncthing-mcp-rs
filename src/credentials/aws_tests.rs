use super::*;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};

#[tokio::test]
async fn test_aws_backend_creation() {
    // Just verify we can create it without crashing
    let _backend = AwsBackend::new("us-east-1".to_string(), None, None).await;
}

#[tokio::test]
async fn test_aws_config_registration() {
    let mut config = crate::config::AppConfig {
        aws: crate::config::AwsConfig {
            enabled: true,
            region: "us-east-1".to_string(),
            profile: None,
            endpoint_url: None,
        },
        ..Default::default()
    };

    // This should register the backend
    config.validate().await.unwrap();
}

#[tokio::test]
async fn test_aws_backend_integration() {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return;
    }

    let node = GenericImage::new("localstack/localstack", "latest")
        .with_wait_for(WaitFor::message_on_stdout("Ready."))
        .with_exposed_port(ContainerPort::Tcp(4566))
        .with_env_var("LOCALSTACK_ACKNOWLEDGE_ACCOUNT_REQUIREMENT", "1")
        .start()
        .await
        .unwrap();

    let host_port = node.get_host_port_ipv4(4566).await.unwrap();
    let address = format!("http://127.0.0.1:{}", host_port);

    // AWS SDK needs some dummy credentials for LocalStack
    unsafe {
        std::env::set_var("AWS_ACCESS_KEY_ID", "test");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test");
        std::env::set_var("AWS_REGION", "us-east-1");
    }

    let backend = AwsBackend::new("us-east-1".to_string(), None, Some(address.clone())).await;

    // Test set
    backend
        .set_api_key("service1", "account1", "key1")
        .await
        .unwrap();

    // Test get
    let key = backend.get_api_key("service1", "account1").await;
    assert_eq!(key, Some("key1".to_string()));

    // Test via AppConfig registration (E2E part)
    let mut config = crate::config::AppConfig {
        aws: crate::config::AwsConfig {
            enabled: true,
            region: "us-east-1".to_string(),
            profile: None,
            endpoint_url: Some(address.clone()),
        },
        instances: vec![crate::config::InstanceConfig {
            name: Some("test-instance".to_string()),
            url: "http://localhost:8384".to_string(),
            api_key: Some("aws:service1:account1".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    // This should register the backend and resolve the API key
    config.validate().await.unwrap();

    assert_eq!(config.instances[0].api_key, Some("key1".to_string()));

    // Test delete
    backend
        .delete_api_key("service1", "account1")
        .await
        .unwrap();
    let key = backend.get_api_key("service1", "account1").await;
    assert_eq!(key, None);
}
