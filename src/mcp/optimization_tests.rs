use crate::mcp::optimization::{alias_fields, filter_fields};
use serde_json::json;

#[test]
fn test_alias_fields() {
    let data = json!({
        "in_sync_bytes": 1024,
        "global_bytes": 2048,
        "state": "idle",
        "nested": {
            "in_sync_bytes": 512
        }
    });

    let mut aliases = std::collections::HashMap::new();
    aliases.insert("in_sync_bytes".to_string(), "isb".to_string());
    aliases.insert("global_bytes".to_string(), "gb".to_string());

    let aliased = alias_fields(data, &aliases);

    assert_eq!(aliased["isb"], 1024);
    assert_eq!(aliased["gb"], 2048);
    assert_eq!(aliased["state"], "idle");
    // Nested fields should also be aliased
    assert_eq!(aliased["nested"]["isb"], 512);
}

#[test]
fn test_filter_fields() {
    let data = json!({
        "id": "abc",
        "label": "test",
        "path": "/tmp/test",
        "paused": false,
        "ignore_patterns": ["*.tmp"]
    });

    let allowed_fields = vec!["id".to_string(), "label".to_string()];
    let filtered = filter_fields(data, &allowed_fields);

    let obj = filtered.as_object().unwrap();
    assert_eq!(obj.len(), 2);
    assert!(obj.contains_key("id"));
    assert!(obj.contains_key("label"));
    assert!(!obj.contains_key("path"));
}
