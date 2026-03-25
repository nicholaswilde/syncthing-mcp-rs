use crate::error::Result;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use serde_json_diff::values;
use serde_json::Value;

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
