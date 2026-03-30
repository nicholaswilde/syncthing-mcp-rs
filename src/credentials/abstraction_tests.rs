use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct MockBackend {
    storage: Mutex<HashMap<String, String>>,
}

impl MockBackend {
    fn new() -> Self {
        Self {
            storage: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CredentialBackend for MockBackend {
    async fn get_api_key(&self, service: &str, account: &str) -> Option<String> {
        let key = format!("{}:{}", service, account);
        self.storage.lock().unwrap().get(&key).cloned()
    }

    async fn set_api_key(&self, service: &str, account: &str, key: &str) -> Result<(), String> {
        let k = format!("{}:{}", service, account);
        self.storage.lock().unwrap().insert(k, key.to_string());
        Ok(())
    }

    async fn delete_api_key(&self, service: &str, account: &str) -> Result<(), String> {
        let k = format!("{}:{}", service, account);
        self.storage.lock().unwrap().remove(&k);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trait_extensions() {
        let backend = MockBackend::new();

        // Test set
        backend
            .set_api_key("service1", "account1", "key1")
            .await
            .unwrap();
        assert_eq!(
            backend.get_api_key("service1", "account1").await,
            Some("key1".to_string())
        );

        // Test delete
        backend
            .delete_api_key("service1", "account1")
            .await
            .unwrap();
        assert_eq!(backend.get_api_key("service1", "account1").await, None);
    }

    #[tokio::test]
    async fn test_backend_registry() {
        let backend = Arc::new(MockBackend::new());
        backend
            .set_api_key("test-service", "test-account", "secret-key")
            .await
            .unwrap();

        register_backend("mock", backend);

        let resolved = resolve_api_key(Some("mock:test-service:test-account".to_string())).await;
        assert_eq!(resolved, Some("secret-key".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_invalid_prefix() {
        let resolved = resolve_api_key(Some("unknown:service:account".to_string())).await;
        assert_eq!(resolved, Some("unknown:service:account".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_invalid_format() {
        let resolved = resolve_api_key(Some("keyring:too-few-parts".to_string())).await;
        assert_eq!(resolved, Some("keyring:too-few-parts".to_string()));
    }
}
