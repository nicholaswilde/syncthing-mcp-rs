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

    #[test]
    fn test_diagnose_french() {
        use crate::error::Language;
        let err = Error::Unauthorized("unauthorized".to_string());
        let diagnostic = err.diagnose_with_language(Language::French);
        assert_eq!(diagnostic.category, "Permission");
        assert!(diagnostic.advice.contains("clé API"));

        let err = Error::Forbidden("CSRF Error".to_string());
        let diagnostic = err.diagnose_with_language(Language::French);
        assert!(diagnostic.advice.contains("CSRF"));

        let err = Error::NotFound("not found".to_string());
        let diagnostic = err.diagnose_with_language(Language::French);
        assert!(diagnostic.advice.contains("Vérifiez l'ID"));

        let err = Error::Network("Connection refused".to_string());
        let diagnostic = err.diagnose_with_language(Language::French);
        assert!(diagnostic.advice.contains("SyncThing ne fonctionne pas"));

        let err = Error::SyncThing("folder \"abc\" not found".to_string());
        let diagnostic = err.diagnose_with_language(Language::French);
        assert!(
            diagnostic
                .advice
                .contains("ID du dossier spécifié est incorrect")
        );

        let err = Error::SyncThing("disk space".to_string());
        let diagnostic = err.diagnose_with_language(Language::French);
        assert!(diagnostic.advice.contains("espace disque"));
    }

    #[test]
    fn test_diagnose_contextual_not_found() {
        let err = Error::NotFound("not found".to_string());
        let contextual_err = Error::Context(Box::new(err), "manage_folders".to_string());
        let diagnostic = contextual_err.diagnose();
        assert!(diagnostic.advice.contains("Confirm the folder ID exists"));

        let err = Error::NotFound("not found".to_string());
        let contextual_err = Error::Context(Box::new(err), "manage_devices".to_string());
        let diagnostic = contextual_err.diagnose();
        assert!(diagnostic.advice.contains("Confirm the device ID exists"));
    }

    #[test]
    fn test_diagnose_path_too_long() {
        let err = Error::SyncThing("filename too long".to_string());
        let diagnostic = err.diagnose();
        assert_eq!(diagnostic.category, "Resource");
        assert!(diagnostic.advice.contains("Windows MAX_PATH"));
    }
}
