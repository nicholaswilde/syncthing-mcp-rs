use crate::error::Result;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

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

/// Generates a diff between the original and conflict versions in the specified format.
pub fn get_diff(original: &str, conflict: &str, format: DiffFormat) -> Result<String> {
    match format {
        DiffFormat::Text => Ok(get_text_diff(original, conflict)),
        DiffFormat::Auto => {
            // Simple heuristic for now: if it looks like JSON or YAML, we'll implement it later.
            // For now, default to text.
            Ok(get_text_diff(original, conflict))
        }
        _ => todo!("implement other formats"),
    }
}
