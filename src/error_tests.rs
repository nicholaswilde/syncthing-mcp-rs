#[cfg(test)]
mod tests {
    use crate::error::Error;

    #[tokio::test]
    async fn test_error_mapping_unauthorized() {
        let res = reqwest::get("http://httpbin.org/status/401").await.unwrap();
        let err = Error::from(res.error_for_status().unwrap_err());
        assert!(matches!(err, Error::Unauthorized(_)));
    }

    #[tokio::test]
    async fn test_error_mapping_forbidden() {
        let res = reqwest::get("http://httpbin.org/status/403").await.unwrap();
        let err = Error::from(res.error_for_status().unwrap_err());
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_error_mapping_not_found() {
        let res = reqwest::get("http://httpbin.org/status/404").await.unwrap();
        let err = Error::from(res.error_for_status().unwrap_err());
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[test]
    fn test_response_error_conversion() {
        use crate::mcp::ResponseError;
        let err = Error::Unauthorized("test".to_string());
        let resp_err = ResponseError::from(err);
        assert_eq!(resp_err.code, -32001);
        let data = resp_err.data.expect("Diagnostic data should be present");
        assert_eq!(data["category"], "Permission");
    }

    #[test]
    fn test_yaml_error_mapping() {
        let err: serde_yaml_ng::Error =
            serde_yaml_ng::from_str::<serde_json::Value>("invalid: : yaml").unwrap_err();
        let error = Error::from(err);
        assert!(matches!(error, Error::Yaml(_)));
    }

    #[test]
    fn test_language_parse() {
        use crate::error::Language;
        assert_eq!(Language::parse("en"), Language::English);
        assert_eq!(Language::parse("fr"), Language::French);
        assert_eq!(Language::parse("FRENCH"), Language::French);
        assert_eq!(Language::parse("unknown"), Language::English);
    }

    #[test]
    fn test_error_from_string() {
        let err = Error::from("test error".to_string());
        assert!(matches!(err, Error::SyncThing(_)));
    }

    #[test]
    fn test_error_diagnose_fr() {
        use crate::error::Language;
        let err = Error::Unauthorized("auth failed".to_string());
        let diag = err.diagnose_with_language(Language::French);
        assert_eq!(diag.explanation, "L'authentification a échoué.");
    }

    #[test]
    fn test_error_diagnose_syncthing_disk_space() {
        let err = Error::SyncThing("no space left on device".to_string());
        let diag = err.diagnose();
        assert_eq!(diag.category, "Resource");
        assert!(diag.advice.contains("Check disk space"));
    }

    #[test]
    fn test_error_diagnose_syncthing_path_too_long() {
        let err = Error::SyncThing("path too long".to_string());
        let diag = err.diagnose();
        assert!(diag.advice.contains("Windows MAX_PATH"));
    }

    #[test]
    fn test_error_diagnose_network_timeout() {
        let err = Error::Network("deadline exceeded".to_string());
        let diag = err.diagnose();
        assert!(
            diag.advice
                .contains("Check if the server is under heavy load")
        );
    }
}
