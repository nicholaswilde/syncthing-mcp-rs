use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use serde_json_diff::values;
use similar::{ChangeTag, TextDiff};
use std::path::Path;

/// The format of the files to diff.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiffFormat {
    /// Automatic detection.
    #[default]
    Auto,
    /// Plain text.
    Text,
    /// JSON structure.
    Json,
    /// YAML structure.
    Yaml,
}

/// Generates a unified diff between two strings.
pub fn get_text_diff(original: &str, conflict: &str) -> String {
    let mut diff_text = String::new();
    let diff = TextDiff::from_lines(original, conflict);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        diff_text.push_str(&format!("{}{}", sign, change));
    }
    diff_text
}

/// Generates a JSON structural diff.
pub fn get_json_diff(original: &str, conflict: &str) -> Result<String> {
    let original_val: Value = serde_json::from_str(original)?;
    let conflict_val: Value = serde_json::from_str(conflict)?;

    if let Some(diff) = values(original_val, conflict_val) {
        Ok(serde_json::to_string_pretty(&diff)?)
    } else {
        Ok("No changes detected in JSON structure.".to_string())
    }
}

/// Generates a YAML structural diff.
pub fn get_yaml_diff(original: &str, conflict: &str) -> Result<String> {
    let original_val: Value = serde_yaml_ng::from_str(original)?;
    let conflict_val: Value = serde_yaml_ng::from_str(conflict)?;

    if let Some(diff) = values(original_val, conflict_val) {
        Ok(serde_json::to_string_pretty(&diff)?)
    } else {
        Ok("No changes detected in YAML structure.".to_string())
    }
}

/// Generates a diff between two strings based on the specified format.
#[allow(clippy::collapsible_if)]
pub fn get_diff(original: &str, conflict: &str, format: DiffFormat) -> Result<String> {
    match format {
        DiffFormat::Text => Ok(get_text_diff(original, conflict)),
        DiffFormat::Json => get_json_diff(original, conflict),
        DiffFormat::Yaml => get_yaml_diff(original, conflict),
        DiffFormat::Auto => {
            // Check if it's likely JSON or YAML
            let is_json =
                original.trim_start().starts_with('{') || original.trim_start().starts_with('[');
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

/// Tool to compare original and conflict versions of a file.
pub async fn diff_conflicts(
    _client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let conflict_path = args["conflict_path"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("Missing conflict_path".to_string()))?;
    let format = serde_json::from_value(args["format"].clone()).unwrap_or(DiffFormat::Auto);

    let path = Path::new(conflict_path);
    let parent = path.parent().ok_or_else(|| {
        crate::error::Error::Internal("Conflict path has no parent directory".to_string())
    })?;
    let filename = path.file_name().ok_or_else(|| {
        crate::error::Error::Internal("Conflict path has no filename".to_string())
    })?;

    // Parse filename to get original path
    let info =
        crate::tools::conflicts::parse_conflict_filename(&filename.to_string_lossy(), parent)
            .ok_or_else(|| {
                crate::error::Error::Internal(format!(
                    "Not a valid SyncThing conflict file: {:?}",
                    filename
                ))
            })?;

    let original_content = tokio::fs::read_to_string(&info.original_path)
        .await
        .map_err(|e| {
            crate::error::Error::Internal(format!("Failed to read original file: {}", e))
        })?;
    let conflict_content = tokio::fs::read_to_string(conflict_path)
        .await
        .map_err(|e| {
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

/// Gets a resolution preview for a conflict.
pub fn get_resolution_preview(original: &str, conflict: &str, action: &str) -> String {
    match action {
        "keep_original" => original.to_string(),
        "keep_conflict" => conflict.to_string(),
        _ => "Invalid resolution action".to_string(),
    }
}

/// Tool to preview conflict resolution.
pub async fn preview_conflict_resolution(
    _client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let conflict_path = args["conflict_path"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("Missing conflict_path".to_string()))?;
    let action = args["action"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("Missing action".to_string()))?;

    let path = Path::new(conflict_path);
    let parent = path.parent().ok_or_else(|| {
        crate::error::Error::Internal("Conflict path has no parent directory".to_string())
    })?;
    let filename = path.file_name().ok_or_else(|| {
        crate::error::Error::Internal("Conflict path has no filename".to_string())
    })?;

    // Parse filename to get original path
    let info =
        crate::tools::conflicts::parse_conflict_filename(&filename.to_string_lossy(), parent)
            .ok_or_else(|| {
                crate::error::Error::Internal(format!(
                    "Not a valid SyncThing conflict file: {:?}",
                    filename
                ))
            })?;

    let original_content = tokio::fs::read_to_string(&info.original_path)
        .await
        .map_err(|e| {
            crate::error::Error::Internal(format!("Failed to read original file: {}", e))
        })?;
    let conflict_content = tokio::fs::read_to_string(conflict_path)
        .await
        .map_err(|e| {
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
