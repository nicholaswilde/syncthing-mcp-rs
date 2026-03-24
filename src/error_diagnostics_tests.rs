#[cfg(test)]
mod tests {
    use crate::error::Error;

    #[test]
    fn test_diagnose_unauthorized() {
        let err = Error::Unauthorized("unauthorized".to_string());
        let diagnostic = err.diagnose();
        assert_eq!(diagnostic.category, "Permission");
        assert!(diagnostic.advice.contains("API key"));
    }

    #[test]
    fn test_diagnose_forbidden_csrf() {
        let err = Error::Forbidden("CSRF Error".to_string());
        let diagnostic = err.diagnose();
        assert_eq!(diagnostic.category, "Permission");
        assert!(diagnostic.advice.contains("CSRF"));
    }

    #[test]
    fn test_diagnose_not_found() {
        let err = Error::NotFound("not found".to_string());
        let diagnostic = err.diagnose();
        assert_eq!(diagnostic.category, "Configuration");
        assert!(diagnostic.advice.contains("Verify the ID"));
    }

    #[test]
    fn test_diagnose_syncthing_folder_not_found() {
        let err = Error::SyncThing("folder \"default\" not found".to_string());
        let diagnostic = err.diagnose();
        assert_eq!(diagnostic.category, "Configuration");
        assert!(diagnostic.advice.contains("folder ID is incorrect"));
    }

    #[test]
    fn test_diagnose_network_refused() {
        let err = Error::Network("Connection refused (os error 111)".to_string());
        let diagnostic = err.diagnose();
        assert_eq!(diagnostic.category, "Network");
        assert!(diagnostic.advice.contains("not running"));
    }
}
