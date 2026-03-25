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
        let err: serde_yaml_ng::Error = serde_yaml_ng::from_str::<serde_json::Value>("invalid: : yaml").unwrap_err();
        let error = Error::from(err);
        assert!(matches!(error, Error::Yaml(_)));
    }
}
