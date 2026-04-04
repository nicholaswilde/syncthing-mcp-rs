use serde_json::{Map, Value};
use std::collections::HashMap;

/// Maps long field names to short aliases recursively in a JSON value.
pub fn alias_fields(value: Value, aliases: &HashMap<String, String>) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = Map::new();
            for (k, v) in map {
                let key = aliases.get(&k).unwrap_or(&k).clone();
                new_map.insert(key, alias_fields(v, aliases));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(|v| alias_fields(v, aliases)).collect())
        }
        _ => value,
    }
}

/// Filters a JSON object to include only specified fields.
/// If the value is not an object, it returns the value as is.
/// Does not recurse into nested objects for filtering (only top-level).
pub fn filter_fields(value: Value, allowed: &[String]) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = Map::new();
            for (k, v) in map {
                if allowed.contains(&k) {
                    new_map.insert(k, v);
                }
            }
            Value::Object(new_map)
        }
        _ => value,
    }
}

/// Recursively truncates JSON arrays to a maximum size.
pub fn truncate_value(value: Value, limit: usize) -> Value {
    match value {
        Value::Array(mut arr) => {
            if arr.len() > limit {
                arr.truncate(limit);
            }
            Value::Array(arr.into_iter().map(|v| truncate_value(v, limit)).collect())
        }
        _ => value,
    }
}

/// Common aliases for SyncThing field names to reduce token usage.
pub fn get_standard_aliases() -> HashMap<String, String> {
    let mut m = HashMap::new();
    // Folder Status (snake_case and camelCase)
    m.insert("in_sync_bytes".to_string(), "isb".to_string());
    m.insert("inSyncBytes".to_string(), "isb".to_string());
    m.insert("global_bytes".to_string(), "gb".to_string());
    m.insert("globalBytes".to_string(), "gb".to_string());
    m.insert("need_bytes".to_string(), "nb".to_string());
    m.insert("needBytes".to_string(), "nb".to_string());
    m.insert("in_sync_files".to_string(), "isf".to_string());
    m.insert("inSyncFiles".to_string(), "isf".to_string());
    m.insert("global_files".to_string(), "gf".to_string());
    m.insert("globalFiles".to_string(), "gf".to_string());
    m.insert("need_files".to_string(), "nf".to_string());
    m.insert("needFiles".to_string(), "nf".to_string());
    m.insert("state".to_string(), "st".to_string());
    
    // Device Status
    m.insert("completion".to_string(), "cp".to_string());
    m.insert("connected".to_string(), "con".to_string());
    
    // Folder Stats
    m.insert("last_scan".to_string(), "ls".to_string());
    m.insert("lastScan".to_string(), "ls".to_string());
    m.insert("last_file".to_string(), "lf".to_string());
    m.insert("lastFile".to_string(), "lf".to_string());
    m
}


/// Applies standard optimizations to a JSON value based on arguments.
pub fn optimize_response(mut value: Value, args: &Value) -> Value {
    // 1. Filter fields if requested
    if let Some(fields) = args.get("fields").and_then(|v| v.as_array()) {
        let allowed: Vec<String> = fields.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
        if !allowed.is_empty() {
            value = filter_fields(value, &allowed);
        }
    }

    // 2. Alias fields if requested (default: true for token optimization)
    if args.get("shorten").and_then(|v| v.as_bool()).unwrap_or(true) {
        value = alias_fields(value, &get_standard_aliases());
    }

    // 3. Truncate arrays if limit is provided
    if let Some(limit) = args.get("limit").and_then(|v| v.as_u64()) {
        value = truncate_value(value, limit as usize);
    }

    value
}

