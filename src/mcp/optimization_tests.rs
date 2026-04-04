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

#[test]
fn test_truncate_array() {
    let data = json!([1, 2, 3, 4, 5]);
    let truncated = crate::mcp::optimization::truncate_value(data, 3);

    assert_eq!(truncated.as_array().unwrap().len(), 3);
    assert_eq!(truncated[0], 1);
    assert_eq!(truncated[2], 3);
}

#[test]
fn test_truncate_nested_arrays() {
    let data = json!({
        "items": [1, 2, 3, 4, 5],
        "other": "value"
    });
    let truncated = crate::mcp::optimization::truncate_value(data, 2);

    assert_eq!(truncated["items"].as_array().unwrap().len(), 2);
    assert_eq!(truncated["other"], "value");
}

#[test]
fn test_token_reduction_benchmark() {
    let aliases = crate::mcp::optimization::get_standard_aliases();
    let raw_data = json!({
        "status": {
            "state": "idle",
            "globalBytes": 104857600,
            "inSyncBytes": 52428800,
            "needBytes": 52428800,
            "globalFiles": 1000,
            "inSyncFiles": 500,
            "needFiles": 500
        },
        "stats": {
            "lastSeen": "2023-01-01T12:00:00Z",
            "lastConnectionDurationS": 3600.0
        }
    });

    let raw_str = serde_json::to_string(&raw_data).unwrap();
    let optimized_data = alias_fields(raw_data.clone(), &aliases);
    let opt_str = serde_json::to_string(&optimized_data).unwrap();

    let filtered_data = crate::mcp::optimization::filter_fields(
        optimized_data,
        &[
            "status".to_string(),
            "st".to_string(),
            "isb".to_string(),
            "gb".to_string(),
        ],
    );
    let filt_str = serde_json::to_string(&filtered_data).unwrap();

    let reduction = (1.0 - (opt_str.len() as f64 / raw_str.len() as f64)) * 100.0;
    let filt_reduction = (1.0 - (filt_str.len() as f64 / raw_str.len() as f64)) * 100.0;

    println!("\n--- Token Usage Benchmark ---");
    println!("Raw JSON size: {} chars", raw_str.len());
    println!("Optimized (Aliased) size: {} chars", opt_str.len());
    println!("Aliased Reduction: {:.2}%", reduction);
    println!("Optimized (Filtered) size: {} chars", filt_str.len());
    println!("Total Reduction: {:.2}%", filt_reduction);

    // Ensure at least some reduction for the benchmark to pass
    assert!(filt_reduction > 30.0);
}
