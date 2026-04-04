use serde_json::{Value, Map};
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
        Value::Object(map) => {
            let mut new_map = Map::new();
            for (k, v) in map {
                new_map.insert(k, truncate_value(v, limit));
            }
            Value::Object(new_map)
        }
        _ => value,
    }
}
