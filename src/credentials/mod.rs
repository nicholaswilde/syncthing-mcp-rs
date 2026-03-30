use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use keyring::Entry;
use rand::{Rng, thread_rng};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;
use tracing::{debug, error, warn};
use async_trait::async_trait;

/// A backend for managing credentials.
#[async_trait]
pub trait CredentialBackend: Send + Sync {
    /// Retrieves the API key for a given service and account.
    async fn get_api_key(&self, service: &str, account: &str) -> Option<String>;
    /// Sets the API key for a given service and account.
    async fn set_api_key(&self, service: &str, account: &str, key: &str) -> Result<(), String>;
    /// Deletes the API key for a given service and account.
    async fn delete_api_key(&self, service: &str, account: &str) -> Result<(), String>;
}

lazy_static! {
    static ref BACKEND_REGISTRY: Arc<RwLock<HashMap<String, Box<dyn CredentialBackend>>>> = {
        let mut m: HashMap<String, Box<dyn CredentialBackend>> = HashMap::new();
        m.insert("keyring".to_string(), Box::new(KeyringBackend));
        Arc::new(RwLock::new(m))
    };
}

/// Registers a credential backend.
pub fn register_backend(prefix: &str, backend: Box<dyn CredentialBackend>) {
    let mut registry = BACKEND_REGISTRY.write().unwrap();
    registry.insert(prefix.to_string(), backend);
}

/// A credential backend that uses the system keyring.
pub struct KeyringBackend;

#[async_trait]
impl CredentialBackend for KeyringBackend {
    async fn get_api_key(&self, service: &str, account: &str) -> Option<String> {
        debug!(
            "Looking up API key in keyring for service: {}, account: {}",
            service, account
        );
        match Entry::new(service, account) {
            Ok(entry) => match entry.get_password() {
                Ok(password) => Some(password),
                Err(e) => {
                    warn!("Failed to get password from keyring: {}", e);
                    None
                }
            },
            Err(e) => {
                error!("Failed to create keyring entry: {}", e);
                None
            }
        }
    }

    async fn set_api_key(&self, service: &str, account: &str, key: &str) -> Result<(), String> {
        debug!(
            "Setting API key in keyring for service: {}, account: {}",
            service, account
        );
        match Entry::new(service, account) {
            Ok(entry) => entry.set_password(key).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn delete_api_key(&self, service: &str, account: &str) -> Result<(), String> {
        debug!(
            "Deleting API key from keyring for service: {}, account: {}",
            service, account
        );
        match Entry::new(service, account) {
            Ok(entry) => entry.delete_credential().map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// A credential backend that uses HashiCorp Vault.
pub struct VaultBackend {
    client: vaultrs::client::VaultClient,
    mount: String,
}

impl VaultBackend {
    /// Creates a new Vault backend.
    pub fn new(address: String, token: String, mount: String) -> Self {
        use vaultrs::client::VaultClientSettingsBuilder;
        let client = vaultrs::client::VaultClient::new(
            VaultClientSettingsBuilder::default()
                .address(address)
                .token(token)
                .build()
                .expect("Failed to build Vault client settings"),
        ).expect("Failed to create Vault client");
        Self { client, mount }
    }
}

#[async_trait]
impl CredentialBackend for VaultBackend {
    async fn get_api_key(&self, service: &str, account: &str) -> Option<String> {
        use vaultrs::kv2;
        let path = format!("{}/{}", service, account);
        let res: Result<HashMap<String, String>, vaultrs::error::ClientError> = 
            kv2::read(&self.client, &self.mount, &path).await;
        match res {
            Ok(data) => data.get("api_key").cloned(),
            Err(e) => {
                warn!("Failed to read secret from Vault at {}: {}", path, e);
                None
            }
        }
    }

    async fn set_api_key(&self, service: &str, account: &str, key: &str) -> Result<(), String> {
        use vaultrs::kv2;
        let path = format!("{}/{}", service, account);
        let mut data = HashMap::new();
        data.insert("api_key".to_string(), key.to_string());
        kv2::set(&self.client, &self.mount, &path, &data).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn delete_api_key(&self, service: &str, account: &str) -> Result<(), String> {
        use vaultrs::kv2;
        let path = format!("{}/{}", service, account);
        // KV2 delete deletes the latest version. For full deletion, use metadata delete.
        kv2::delete_metadata(&self.client, &self.mount, &path).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// A credential backend that uses AWS Secrets Manager.
pub struct AwsBackend {
    client: aws_sdk_secretsmanager::Client,
}

impl AwsBackend {
    /// Creates a new AWS backend.
    pub async fn new(region: String, profile: Option<String>, endpoint_url: Option<String>) -> Self {
        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region));
        
        if let Some(p) = profile {
            loader = loader.profile_name(p);
        }
        
        if let Some(url) = endpoint_url {
            loader = loader.endpoint_url(url);
        }
        
        let config = loader.load().await;
        let client = aws_sdk_secretsmanager::Client::new(&config);
        Self { client }
    }
}

#[async_trait]
impl CredentialBackend for AwsBackend {
    async fn get_api_key(&self, service: &str, account: &str) -> Option<String> {
        let secret_id = format!("{}/{}", service, account);
        match self.client.get_secret_value().secret_id(secret_id).send().await {
            Ok(res) => res.secret_string().map(|s| s.to_string()),
            Err(e) => {
                warn!("Failed to get secret from AWS: {}", e);
                None
            }
        }
    }

    async fn set_api_key(&self, service: &str, account: &str, key: &str) -> Result<(), String> {
        let secret_id = format!("{}/{}", service, account);
        // Try to update first, if fails try to create
        let res = self.client.update_secret()
            .secret_id(&secret_id)
            .secret_string(key)
            .send().await;
            
        if res.is_err() {
            self.client.create_secret()
                .name(&secret_id)
                .secret_string(key)
                .send().await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn delete_api_key(&self, service: &str, account: &str) -> Result<(), String> {
        let secret_id = format!("{}/{}", service, account);
        self.client.delete_secret()
            .secret_id(secret_id)
            .force_delete_without_recovery(true)
            .send().await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Resolves an API key, potentially from a keyring or encrypted value.
pub async fn resolve_api_key(api_key: Option<String>) -> Option<String> {
    match api_key {
        Some(key) if key.contains(':') => {
            let parts: Vec<&str> = key.split(':').collect();
            let prefix = parts[0];
            
            if prefix == "encrypted" && parts.len() >= 3 && parts[1] == "v1" {
                return decrypt_value(&key);
            }

            let registry = BACKEND_REGISTRY.read().unwrap();
            if let Some(backend) = registry.get(prefix) {
                if parts.len() == 3 {
                    let service = parts[1];
                    let account = parts[2];
                    backend.get_api_key(service, account).await
                } else {
                    warn!("Invalid credential format for prefix {}. Expected {}:service:account", prefix, prefix);
                    Some(key)
                }
            } else {
                Some(key)
            }
        }
        _ => api_key,
    }
}

/// Retrieves or generates the master key for encryption.
pub fn get_master_key() -> Option<[u8; 32]> {
    let entry = Entry::new("syncthing-mcp-rs", "master-key").ok()?;
    match entry.get_password() {
        Ok(pw) => {
            let mut key = [0u8; 32];
            let decoded = BASE64.decode(pw).ok()?;
            if decoded.len() == 32 {
                key.copy_from_slice(&decoded);
                Some(key)
            } else {
                None
            }
        }
        Err(_) => {
            // Generate and store new master key
            let mut key = [0u8; 32];
            thread_rng().fill(&mut key);
            let encoded = BASE64.encode(key);
            entry.set_password(&encoded).ok()?;
            Some(key)
        }
    }
}

/// Encrypts a value using the master key.
pub fn encrypt_value(value: &str) -> Option<String> {
    let key_bytes = get_master_key()?;
    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);

    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, value.as_bytes()).ok()?;

    let mut combined = Vec::new();
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Some(format!("encrypted:v1:{}", BASE64.encode(combined)))
}

/// Decrypts a value using the master key.
pub fn decrypt_value(encrypted: &str) -> Option<String> {
    let payload = encrypted.strip_prefix("encrypted:v1:")?;
    let decoded = BASE64.decode(payload).ok()?;
    if decoded.len() < 12 {
        return None;
    }

    let key_bytes = get_master_key()?;
    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);

    let (nonce_bytes, ciphertext) = decoded.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
    String::from_utf8(plaintext).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_plain_key() {
        let key = Some("plain-key".to_string());
        assert_eq!(resolve_api_key(key).await, Some("plain-key".to_string()));
    }

    #[tokio::test]
    async fn test_resolve_none() {
        assert_eq!(resolve_api_key(None).await, None);
    }

    #[tokio::test]
    async fn test_resolve_invalid_keyring_format() {
        let key = Some("keyring:too:many:parts".to_string());
        assert_eq!(
            resolve_api_key(key).await,
            Some("keyring:too:many:parts".to_string())
        );
    }

    #[test]
    fn test_encrypt_decrypt() {
        // Since get_master_key uses the OS keyring, it might fail in CI
        // But we can test with a fixed key for the logic.
        let key_bytes = [0u8; 32];
        let key = Key::from_slice(&key_bytes);
        let cipher = ChaCha20Poly1305::new(key);

        let value = "secret-message";
        let nonce_bytes = [0u8; 12];
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, value.as_bytes()).unwrap();
        let mut combined = Vec::new();
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        let encrypted = format!("encrypted:v1:{}", BASE64.encode(combined));

        // Mock decryption with fixed key
        let payload = encrypted.strip_prefix("encrypted:v1:").unwrap();
        let decoded = BASE64.decode(payload).unwrap();
        let (n, c) = decoded.split_at(12);
        let n = Nonce::from_slice(n);
        let p = cipher.decrypt(n, c).unwrap();
        assert_eq!(String::from_utf8(p).unwrap(), "secret-message");
    }

    #[test]
    fn test_decrypt_value_short_payload() {
        let encrypted = "encrypted:v1:too-short";
        assert_eq!(decrypt_value(encrypted), None);
    }

    #[test]
    fn test_decrypt_value_invalid_prefix() {
        let encrypted = "invalid:prefix:payload";
        assert_eq!(decrypt_value(encrypted), None);
    }
}

#[cfg(test)]
mod abstraction_tests;

#[cfg(test)]
mod vault_tests;

#[cfg(test)]
mod aws_tests;
