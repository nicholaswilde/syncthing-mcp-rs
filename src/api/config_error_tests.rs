use crate::api::models::*;

#[test]
fn test_parse_config_insync() {
    let json = r#"{"configInSync": true}"#;
    let config_insync: ConfigInSync = serde_json::from_str(json).unwrap();
    assert!(config_insync.insync);
}

#[test]
fn test_parse_system_errors() {
    let json = r#"{
        "errors": [
            {
                "when": "2014-09-18T12:59:26.57076632+02:00",
                "message": "Protocol error: unknown message type 17"
            }
        ]
    }"#;
    let system_errors: SystemErrors = serde_json::from_str(json).unwrap();
    assert_eq!(system_errors.errors.as_ref().unwrap().len(), 1);
    assert_eq!(
        system_errors.errors.as_ref().unwrap()[0].message,
        "Protocol error: unknown message type 17"
    );
}

#[test]
fn test_parse_system_errors_null() {
    let json = r#"{"errors": null}"#;
    let system_errors: SystemErrors = serde_json::from_str(json).unwrap();
    assert!(system_errors.errors.is_none());
}
