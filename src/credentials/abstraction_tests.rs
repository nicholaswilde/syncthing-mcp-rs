use super::*;
use std::collections::HashMap;
use std::sync::Mutex;

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

impl CredentialBackend for MockBackend {
    fn get_api_key(&self, service: &str, account: &str) -> Option<String> {
        let key = format!("{}:{}", service, account);
        self.storage.lock().unwrap().get(&key).cloned()
    }

    fn set_api_key(&self, service: &str, account: &str, key: &str) -> Result<(), String> {
        let k = format!("{}:{}", service, account);
        self.storage.lock().unwrap().insert(k, key.to_string());
        Ok(())
    }

    fn delete_api_key(&self, service: &str, account: &str) -> Result<(), String> {
        let k = format!("{}:{}", service, account);
        self.storage.lock().unwrap().remove(&k);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_extensions() {
        let backend = MockBackend::new();
        
        // Test set
        backend.set_api_key("service1", "account1", "key1").unwrap();
        assert_eq!(backend.get_api_key("service1", "account1"), Some("key1".to_string()));
        
        // Test delete
        backend.delete_api_key("service1", "account1").unwrap();
        assert_eq!(backend.get_api_key("service1", "account1"), None);
    }

    #[test]
    fn test_backend_registry() {
        let backend = MockBackend::new();
        backend.set_api_key("test-service", "test-account", "secret-key").unwrap();
        
        register_backend("mock", Box::new(backend));
        
        let resolved = resolve_api_key(Some("mock:test-service:test-account".to_string()));
        assert_eq!(resolved, Some("secret-key".to_string()));
    }
}
