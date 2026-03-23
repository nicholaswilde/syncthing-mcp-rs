use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use keyring::Entry;
use rand::{Rng, thread_rng};
use tracing::{debug, error, warn};

/// A provider for retrieving credentials.
pub trait CredentialProvider {
    /// Retrieves the API key for a given service and account.
    fn get_api_key(&self, service: &str, account: &str) -> Option<String>;
}

/// A credential provider that uses the system keyring.
pub struct KeyringProvider;

impl CredentialProvider for KeyringProvider {
    fn get_api_key(&self, service: &str, account: &str) -> Option<String> {
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
}

/// Resolves an API key, potentially from a keyring or encrypted value.
pub fn resolve_api_key(api_key: Option<String>) -> Option<String> {
    match api_key {
        Some(key) if key.starts_with("keyring:") => {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 3 {
                let service = parts[1];
                let account = parts[2];
                KeyringProvider.get_api_key(service, account)
            } else {
                warn!("Invalid keyring format. Expected keyring:service:account");
                Some(key)
            }
        }
        Some(key) if key.starts_with("encrypted:v1:") => decrypt_value(&key),
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

    #[test]
    fn test_resolve_plain_key() {
        let key = Some("plain-key".to_string());
        assert_eq!(resolve_api_key(key), Some("plain-key".to_string()));
    }

    #[test]
    fn test_resolve_none() {
        assert_eq!(resolve_api_key(None), None);
    }

    #[test]
    fn test_resolve_invalid_keyring_format() {
        let key = Some("keyring:too:many:parts".to_string());
        assert_eq!(
            resolve_api_key(key),
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
}
