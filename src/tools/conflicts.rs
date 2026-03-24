use crate::error::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;

lazy_static! {
    static ref CONFLICT_REGEX: Regex = Regex::new(
        r"^(?P<base>.*)\.sync-conflict-(?P<timestamp>\d{8}-\d{6})-(?P<device>[A-Z0-9]+)(\.(?P<ext>.*))?$"
    ).unwrap();
}

/// Information about a conflict file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ConflictInfo {
    /// Path to the conflict file.
    pub conflict_path: String,
    /// Path to the original file.
    pub original_path: String,
    /// The timestamp from the conflict filename.
    pub timestamp: String,
    /// The device ID from the conflict filename.
    pub device_id: String,
}

/// Scans a directory for SyncThing conflict files.
pub async fn scan_conflicts(path: &Path) -> Result<Vec<ConflictInfo>> {
    let mut conflicts = Vec::new();
    if !path.is_dir() {
        return Ok(conflicts);
    }

    let mut dir = tokio::fs::read_dir(path).await.map_err(|e| {
        crate::error::Error::Internal(format!("Failed to read directory: {}", e))
    })?;

    while let Some(entry) = dir.next_entry().await.map_err(|e| {
        crate::error::Error::Internal(format!("Failed to read directory entry: {}", e))
    })? {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        if let Some(info) = parse_conflict_filename(&file_name_str, path) {
            conflicts.push(info);
        }
    }

    Ok(conflicts)
}

fn parse_conflict_filename(filename: &str, parent: &Path) -> Option<ConflictInfo> {
    let caps = CONFLICT_REGEX.captures(filename)?;
    let base = caps.name("base")?.as_str();
    let timestamp = caps.name("timestamp")?.as_str();
    let device_id = caps.name("device")?.as_str();
    let ext = caps.name("ext").map(|m| m.as_str());

    let original_filename = if let Some(e) = ext {
        format!("{}.{}", base, e)
    } else {
        base.to_string()
    };

    let conflict_path = parent.join(filename);
    let original_path = parent.join(original_filename);

    Some(ConflictInfo {
        conflict_path: conflict_path.to_string_lossy().to_string(),
        original_path: original_path.to_string_lossy().to_string(),
        timestamp: timestamp.to_string(),
        device_id: device_id.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_regex_parsing() {
        let parent = Path::new("/tmp");
        let filename = "notes.sync-conflict-20230101-120000-ABCDEFG.txt";
        let info = parse_conflict_filename(filename, parent).unwrap();
        assert_eq!(info.timestamp, "20230101-120000");
        assert_eq!(info.device_id, "ABCDEFG");
        assert_eq!(info.original_path, "/tmp/notes.txt");
        assert_eq!(info.conflict_path, "/tmp/notes.sync-conflict-20230101-120000-ABCDEFG.txt");
    }

    #[test]
    fn test_conflict_regex_parsing_no_ext() {
        let parent = Path::new("/tmp");
        let filename = "README.sync-conflict-20230101-120000-ABCDEFG";
        let info = parse_conflict_filename(filename, parent).unwrap();
        assert_eq!(info.timestamp, "20230101-120000");
        assert_eq!(info.device_id, "ABCDEFG");
        assert_eq!(info.original_path, "/tmp/README");
    }
}
