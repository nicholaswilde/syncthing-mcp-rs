use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use serde_json_diff::values;
use serde_json::{Value, json};
use std::path::Path;

/// The format of the files to diff.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum DiffFormat {
    /// Plain text format.
    Text,
    /// JSON format.
    Json,
    /// YAML format.
    Yaml,
    /// Automatically detect format.
    Auto,
}

impl From<&str> for DiffFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => DiffFormat::Json,
            "yaml" | "yml" => DiffFormat::Yaml,
            "text" | "txt" => DiffFormat::Text,
            _ => DiffFormat::Auto,
        }
    }
}

/// Generates a textual diff between the original and conflict versions.
pub fn get_text_diff(original: &str, conflict: &str) -> String {
    let diff = TextDiff::from_lines(original, conflict);
    let mut result = String::new();

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        result.push_str(&format!("{}{}", sign, change));
    }
    result
}

/// Generates a semantic JSON diff between the original and conflict versions.
pub fn get_json_diff(original: &str, conflict: &str) -> Result<String> {
    let original_val: Value = serde_json::from_str(original)?;
    let conflict_val: Value = serde_json::from_str(conflict)?;
    
    if let Some(diff) = values(original_val, conflict_val) {
        Ok(serde_json::to_string_pretty(&diff)?)
    } else {
        Ok("No changes detected in JSON structure.".to_string())
    }
}

/// Generates a semantic YAML diff between the original and conflict versions.
pub fn get_yaml_diff(original: &str, conflict: &str) -> Result<String> {
    let original_val: Value = serde_yaml_ng::from_str(original)?;
    let conflict_val: Value = serde_yaml_ng::from_str(conflict)?;
    
    if let Some(diff) = values(original_val, conflict_val) {
        Ok(serde_json::to_string_pretty(&diff)?)
    } else {
        Ok("No changes detected in YAML structure.".to_string())
    }
}

/// Generates a preview of the conflict resolution.
pub fn get_resolution_preview(original: &str, conflict: &str, action: &str) -> String {
    match action {
        "keep_original" => original.to_string(),
        "keep_conflict" => conflict.to_string(),
        _ => "Unsupported action".to_string(),
    }
}

/// Generates a diff between the original and conflict versions in the specified format.
pub fn get_diff(original: &str, conflict: &str, format: DiffFormat) -> Result<String> {
    match format {
        DiffFormat::Text => Ok(get_text_diff(original, conflict)),
        DiffFormat::Json => get_json_diff(original, conflict),
        DiffFormat::Yaml => get_yaml_diff(original, conflict),
        DiffFormat::Auto => {
            // Check if it's likely JSON or YAML
            let is_json = original.trim_start().starts_with('{') || original.trim_start().starts_with('[');
            // YAML is harder to detect but we'll try it if it looks structured
            let is_yaml = original.contains(": ");

            if is_json {
                if let Ok(diff) = get_json_diff(original, conflict) {
                    if diff != "No changes detected in JSON structure." {
                        return Ok(diff);
                    }
                }
            }

            if is_yaml {
                if let Ok(diff) = get_yaml_diff(original, conflict) {
                     if diff != "No changes detected in YAML structure." {
                        return Ok(diff);
                    }
                }
            }

            // Fallback to text
            Ok(get_text_diff(original, conflict))
        }
    }
}

/// MCP tool for diffing conflict files.
pub async fn diff_conflicts(
    _client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let conflict_path_str = args["conflict_path"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("conflict_path is required".to_string()))?;
    let format_str = args["format"].as_str().unwrap_or("auto");
    let format = DiffFormat::from(format_str);

    let conflict_path = Path::new(conflict_path_str);
    let filename = conflict_path.file_name().ok_or_else(|| {
        crate::error::Error::Internal("Invalid conflict_path: no filename".to_string())
    })?;
    let parent = conflict_path.parent().ok_or_else(|| {
        crate::error::Error::Internal("Invalid conflict_path: no parent directory".to_string())
    })?;

    // Parse filename to get original path
    let info = crate::tools::conflicts::parse_conflict_filename(&filename.to_string_lossy(), parent).ok_or_else(|| {
        crate::error::Error::Internal(format!("Not a valid SyncThing conflict file: {:?}", filename))
    })?;

    let original_content = tokio::fs::read_to_string(&info.original_path).await.map_err(|e| {
        crate::error::Error::Internal(format!("Failed to read original file: {}", e))
    })?;
    let conflict_content = tokio::fs::read_to_string(conflict_path).await.map_err(|e| {
        crate::error::Error::Internal(format!("Failed to read conflict file: {}", e))
    })?;

    let diff = get_diff(&original_content, &conflict_content, format)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": diff
        }]
    }))
}

/// MCP tool for previewing conflict resolution.
pub async fn preview_conflict_resolution(
    _client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let conflict_path_str = args["conflict_path"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("conflict_path is required".to_string()))?;
    let action = args["action"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("action is required".to_string()))?;

    let conflict_path = Path::new(conflict_path_str);
    let filename = conflict_path.file_name().ok_or_else(|| {
        crate::error::Error::Internal("Invalid conflict_path: no filename".to_string())
    })?;
    let parent = conflict_path.parent().ok_or_else(|| {
        crate::error::Error::Internal("Invalid conflict_path: no parent directory".to_string())
    })?;

    // Parse filename to get original path
    let info = crate::tools::conflicts::parse_conflict_filename(&filename.to_string_lossy(), parent).ok_or_else(|| {
        crate::error::Error::Internal(format!("Not a valid SyncThing conflict file: {:?}", filename))
    })?;

    let original_content = tokio::fs::read_to_string(&info.original_path).await.map_err(|e| {
        crate::error::Error::Internal(format!("Failed to read original file: {}", e))
    })?;
    let conflict_content = tokio::fs::read_to_string(conflict_path).await.map_err(|e| {
        crate::error::Error::Internal(format!("Failed to read conflict file: {}", e))
    })?;

    let preview = get_resolution_preview(&original_content, &conflict_content, action);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": preview
        }]
    }))
}
