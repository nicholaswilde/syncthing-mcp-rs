use keyring::Entry;
use tracing::{debug, error, warn};

pub trait CredentialProvider {
    fn get_api_key(&self, service: &str, account: &str) -> Option<String>;
}

pub struct KeyringProvider;

impl CredentialProvider for KeyringProvider {
    fn get_api_key(&self, service: &str, account: &str) -> Option<String> {
        debug!("Looking up API key in keyring for service: {}, account: {}", service, account);
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
        _ => api_key,
    }
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
        assert_eq!(resolve_api_key(key), Some("keyring:too:many:parts".to_string()));
    }
}
