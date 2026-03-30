use super::*;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};

#[tokio::test]
async fn test_vault_backend_integration() {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return;
    }

    let node = GenericImage::new("hashicorp/vault", "1.15")
        .with_wait_for(WaitFor::message_on_stdout("Vault server started!"))
        .with_exposed_port(ContainerPort::Tcp(8200))
        .with_env_var("VAULT_DEV_ROOT_TOKEN_ID", "root")
        .with_env_var("VAULT_DEV_LISTEN_ADDRESS", "0.0.0.0:8200")
        .start()
        .await
        .unwrap();

    let host_port = node.get_host_port_ipv4(8200).await.unwrap();
    let address = format!("http://127.0.0.1:{}", host_port);

    let backend = VaultBackend::new(address, "root".to_string(), "secret".to_string());
    
    // Test set
    backend.set_api_key("service1", "account1", "key1").await.unwrap();
    
    // Test get
    let key = backend.get_api_key("service1", "account1").await;
    assert_eq!(key, Some("key1".to_string()));
    
    // Test delete
    backend.delete_api_key("service1", "account1").await.unwrap();
    
    // KV2 read returns error if not found (usually 404)
    let key = backend.get_api_key("service1", "account1").await;
    assert_eq!(key, None);
}
