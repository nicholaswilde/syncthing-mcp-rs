#[cfg(test)]
mod tests {
    use crate::credentials::{CredentialBackend, decrypt_value, register_backend, resolve_api_key};
    use async_trait::async_trait;
    use std::sync::Arc;

    struct MockBackend;

    #[async_trait]
    impl CredentialBackend for MockBackend {
        async fn get_api_key(&self, service: &str, account: &str) -> Option<String> {
            if service == "test-service" && account == "test-account" {
                Some("secret-api-key".to_string())
            } else {
                None
            }
        }
        async fn set_api_key(
            &self,
            _service: &str,
            _account: &str,
            _key: &str,
        ) -> Result<(), String> {
            Ok(())
        }
        async fn delete_api_key(&self, _service: &str, _account: &str) -> Result<(), String> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_resolve_mock_backend() {
        register_backend("mock", Arc::new(MockBackend));

        let key = Some("mock:test-service:test-account".to_string());
        assert_eq!(
            resolve_api_key(key).await,
            Some("secret-api-key".to_string())
        );

        let key_not_found = Some("mock:other:other".to_string());
        assert_eq!(resolve_api_key(key_not_found).await, None);

        let key_invalid_format = Some("mock:only-one-part".to_string());
        assert_eq!(
            resolve_api_key(key_invalid_format).await,
            Some("mock:only-one-part".to_string())
        );
    }

    #[test]
    fn test_decrypt_value_invalid_base64() {
        let encrypted = "encrypted:v1:not-base64-!@#$";
        assert_eq!(decrypt_value(encrypted), None);
    }
}
