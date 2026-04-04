use serde_json::json;
use std::collections::HashMap;

fn main() {
    let mut aliases = HashMap::new();
    aliases.insert("in_sync_bytes".to_string(), "isb".to_string());
    aliases.insert("inSyncBytes".to_string(), "isb".to_string());
    aliases.insert("global_bytes".to_string(), "gb".to_string());
    aliases.insert("globalBytes".to_string(), "gb".to_string());
    aliases.insert("need_bytes".to_string(), "nb".to_string());
    aliases.insert("needBytes".to_string(), "nb".to_string());
    aliases.insert("state".to_string(), "st".to_string());
    aliases.insert("last_seen".to_string(), "ls".to_string());
    aliases.insert("lastSeen".to_string(), "ls".to_string());

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
    let raw_pretty = serde_json::to_string_pretty(&raw_data).unwrap();

    let mut optimized_data = raw_data.clone();
    // Apply aliasing manually for the script
    optimized_data = alias_fields(optimized_data, &aliases);

    let opt_str = serde_json::to_string(&optimized_data).unwrap();
    let opt_pretty = serde_json::to_string_pretty(&optimized_data).unwrap();

    println!("--- Token Usage Benchmark ---");
    println!("Raw JSON size: {} chars", raw_str.len());
    println!("Optimized JSON size: {} chars", opt_str.len());
    println!("Reduction: {:.2}%", (1.0 - (opt_str.len() as f64 / raw_str.len() as f64)) * 100.0);
    println!("\nPretty Raw JSON size: {} chars", raw_pretty.len());
    println!("Pretty Optimized JSON size: {} chars", opt_pretty.len());
    println!("Pretty Reduction: {:.2}%", (1.0 - (opt_pretty.len() as f64 / raw_pretty.len() as f64)) * 100.0);
}

fn alias_fields(value: serde_json::Value, aliases: &HashMap<String, String>) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                let key = aliases.get(&k).unwrap_or(&k).clone();
                new_map.insert(key, alias_fields(v, aliases));
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(|v| alias_fields(v, aliases)).collect())
        }
        _ => value,
    }
}
