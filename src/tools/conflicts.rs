use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{Value, json};
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
    /// Size of the conflict file in bytes.
    pub conflict_size: u64,
    /// Size of the original file in bytes (if it exists).
    pub original_size: Option<u64>,
    /// Modification time of the conflict file.
    pub conflict_modified: String,
    /// Modification time of the original file (if it exists).
    pub original_modified: Option<String>,
}

/// Scans a directory and its subdirectories for SyncThing conflict files.
pub async fn scan_conflicts(path: &Path) -> Result<Vec<ConflictInfo>> {
    let mut conflicts = Vec::new();
    scan_recursive(path, path, &mut conflicts).await?;
    Ok(conflicts)
}

#[async_recursion::async_recursion]
async fn scan_recursive(
    _root: &Path,
    current: &Path,
    conflicts: &mut Vec<ConflictInfo>,
) -> Result<()> {
    if !current.is_dir() {
        return Ok(());
    }

    let mut dir = tokio::fs::read_dir(current)
        .await
        .map_err(|e| crate::error::Error::Internal(format!("Failed to read directory: {}", e)))?;

    while let Some(entry) = dir.next_entry().await.map_err(|e| {
        crate::error::Error::Internal(format!("Failed to read directory entry: {}", e))
    })? {
        let file_type = entry.file_type().await.map_err(|e| {
            crate::error::Error::Internal(format!("Failed to get file type: {}", e))
        })?;

        if file_type.is_dir() {
            scan_recursive(_root, &entry.path(), conflicts).await?;
        } else {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if let Some(mut info) = parse_conflict_filename(&file_name_str, current) {
                // Fetch metadata
                if let Ok(metadata) = entry.metadata().await {
                    info.conflict_size = metadata.len();
                    info.conflict_modified = format_system_time(metadata.modified().ok());
                }

                let original_path = Path::new(&info.original_path);
                if let Ok(metadata) = tokio::fs::metadata(original_path).await {
                    info.original_size = Some(metadata.len());
                    info.original_modified = Some(format_system_time(metadata.modified().ok()));
                }

                conflicts.push(info);
            }
        }
    }

    Ok(())
}

fn format_system_time(time: Option<std::time::SystemTime>) -> String {
    time.map(|t| {
        let datetime: chrono::DateTime<chrono::Utc> = t.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    })
    .unwrap_or_else(|| "Unknown".to_string())
}

pub(crate) fn parse_conflict_filename(filename: &str, parent: &Path) -> Option<ConflictInfo> {
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
        conflict_size: 0,
        original_size: None,
        conflict_modified: "Unknown".to_string(),
        original_modified: None,
    })
}

/// Lists SyncThing conflict files.
pub async fn list_conflicts(
    client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let folder_id = args["folder_id"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("folder_id is required".to_string()))?;

    let folder = client.get_folder(folder_id).await?;
    let path = Path::new(&folder.path);

    let conflicts = scan_conflicts(path).await?;

    if conflicts.is_empty() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("No conflicts found in folder {}.", folder_id)
            }]
        }));
    }

    let mut text = format!("Conflicts in folder {}:\n", folder_id);
    for conflict in conflicts {
        let conflict_file = Path::new(&conflict.conflict_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let original_file = Path::new(&conflict.original_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        text.push_str(&format!("- {}\n", conflict_file));
        text.push_str(&format!(
            "  - Conflict: Size: {} bytes, Modified: {}\n",
            conflict.conflict_size, conflict.conflict_modified
        ));

        if let Some(size) = conflict.original_size {
            text.push_str(&format!(
                "  - Original: {} (Size: {} bytes, Modified: {})\n",
                original_file,
                size,
                conflict
                    .original_modified
                    .unwrap_or_else(|| "Unknown".to_string())
            ));
        } else {
            text.push_str(&format!("  - Original: {} (NOT FOUND)\n", original_file));
        }
        text.push_str(&format!(
            "  - Details: Device: {}, Conflict Time: {}\n",
            conflict.device_id, conflict.timestamp
        ));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }]
    }))
}

/// Resolves a SyncThing conflict file.
pub async fn resolve_conflict(
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
    let parent = conflict_path.parent().ok_or_else(|| {
        crate::error::Error::Internal("Invalid conflict_path: no parent directory".to_string())
    })?;
    let filename = conflict_path
        .file_name()
        .ok_or_else(|| {
            crate::error::Error::Internal("Invalid conflict_path: no filename".to_string())
        })?
        .to_string_lossy();

    let info = parse_conflict_filename(&filename, parent).ok_or_else(|| {
        crate::error::Error::Internal(format!("Not a valid SyncThing conflict file: {}", filename))
    })?;

    let dry_run = args["dry_run"].as_bool().unwrap_or(false);
    let backup = args["backup"].as_bool().unwrap_or(true);
    let preview = args["preview"].as_bool().unwrap_or(false);

    if preview {
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

        let preview_text = crate::tools::diff::get_resolution_preview(
            &original_content,
            &conflict_content,
            action,
        );
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("PREVIEW of resolution: {}\n\nResulting content:\n{}", action, preview_text)
            }]
        }));
    }

    match action {
        "keep_original" => {
            if dry_run {
                return Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("[DRY RUN] Would resolve conflict by keeping original version (delete {})", filename)
                    }]
                }));
            }
            if backup {
                trash::delete(&info.conflict_path).map_err(|e| {
                    crate::error::Error::Internal(format!(
                        "Failed to move conflict file to trash: {}",
                        e
                    ))
                })?;
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Resolved conflict by keeping original version (moved {} to trash)", filename)
                    }]
                }))
            } else {
                tokio::fs::remove_file(&info.conflict_path)
                    .await
                    .map_err(|e| {
                        crate::error::Error::Internal(format!(
                            "Failed to delete conflict file: {}",
                            e
                        ))
                    })?;
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Resolved conflict by keeping original version (deleted {})", filename)
                    }]
                }))
            }
        }
        "keep_conflict" => {
            if dry_run {
                return Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("[DRY RUN] Would resolve conflict by keeping conflict version (replace original with {})", filename)
                    }]
                }));
            }
            if backup {
                // If original file doesn't exist, we don't need to trash it
                if Path::new(&info.original_path).exists() {
                    trash::delete(&info.original_path).map_err(|e| {
                        crate::error::Error::Internal(format!(
                            "Failed to move original file to trash: {}",
                            e
                        ))
                    })?;
                }
            }
            tokio::fs::rename(&info.conflict_path, &info.original_path)
                .await
                .map_err(|e| {
                    crate::error::Error::Internal(format!(
                        "Failed to replace original file with conflict file: {}",
                        e
                    ))
                })?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Resolved conflict by keeping conflict version (replaced original with {})", filename)
                }]
            }))
        }
        _ => Err(crate::error::Error::Internal(format!(
            "Unsupported action: {}",
            action
        ))),
    }
}

/// Deletes a SyncThing conflict file.
pub async fn delete_conflict(
    _client: SyncThingClient,
    _config: AppConfig,
    args: Value,
) -> Result<Value> {
    let conflict_path_str = args["conflict_path"]
        .as_str()
        .ok_or_else(|| crate::error::Error::Internal("conflict_path is required".to_string()))?;

    let conflict_path = Path::new(conflict_path_str);
    let parent = conflict_path.parent().ok_or_else(|| {
        crate::error::Error::Internal("Invalid conflict_path: no parent directory".to_string())
    })?;
    let filename = conflict_path
        .file_name()
        .ok_or_else(|| {
            crate::error::Error::Internal("Invalid conflict_path: no filename".to_string())
        })?
        .to_string_lossy();

    // Validate that it is indeed a conflict file
    if parse_conflict_filename(&filename, parent).is_none() {
        return Err(crate::error::Error::Internal(format!(
            "Not a valid SyncThing conflict file: {}",
            filename
        )));
    }

    let dry_run = args["dry_run"].as_bool().unwrap_or(false);
    let backup = args["backup"].as_bool().unwrap_or(true);

    if dry_run {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("[DRY RUN] Would delete conflict file: {}", filename)
            }]
        }));
    }

    if backup {
        trash::delete(conflict_path).map_err(|e| {
            crate::error::Error::Internal(format!("Failed to move conflict file to trash: {}", e))
        })?;
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Moved conflict file to trash: {}", filename)
            }]
        }))
    } else {
        tokio::fs::remove_file(conflict_path).await.map_err(|e| {
            crate::error::Error::Internal(format!("Failed to delete conflict file: {}", e))
        })?;
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Deleted conflict file: {}", filename)
            }]
        }))
    }
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
        assert_eq!(
            info.conflict_path,
            "/tmp/notes.sync-conflict-20230101-120000-ABCDEFG.txt"
        );
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

    #[test]
    fn test_conflict_regex_parsing_multiple_dots() {
        let parent = Path::new("/tmp");
        let filename = "archive.tar.gz.sync-conflict-20230101-120000-ABCDEFG.bak";
        let info = parse_conflict_filename(filename, parent).unwrap();
        assert_eq!(info.original_path, "/tmp/archive.tar.gz.bak");
    }

    #[test]
    fn test_conflict_regex_parsing_invalid() {
        let parent = Path::new("/tmp");
        assert!(parse_conflict_filename("not-a-conflict.txt", parent).is_none());
        assert!(parse_conflict_filename("file.sync-conflict-invalid-DEVICE.txt", parent).is_none());
    }

    #[tokio::test]
    async fn test_scan_conflicts() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path();

        // Create original file
        let original_path = path.join("test.txt");
        tokio::fs::write(&original_path, "original content")
            .await
            .unwrap();

        // Create conflict file
        let conflict_name = "test.sync-conflict-20230101-120000-ABCDEFG.txt";
        let conflict_path = path.join(conflict_name);
        tokio::fs::write(&conflict_path, "conflict content")
            .await
            .unwrap();

        // Create sub-directory with conflict
        let sub = path.join("sub");
        tokio::fs::create_dir(&sub).await.unwrap();
        let sub_original = sub.join("notes.md");
        tokio::fs::write(&sub_original, "sub original")
            .await
            .unwrap();
        let sub_conflict_name = "notes.sync-conflict-20230101-120000-ABCDEFG.md";
        let sub_conflict_path = sub.join(sub_conflict_name);
        tokio::fs::write(&sub_conflict_path, "sub conflict")
            .await
            .unwrap();

        let conflicts = scan_conflicts(path).await.unwrap();
        assert_eq!(conflicts.len(), 2);

        // Sort by path for stable assertion
        let mut conflicts = conflicts;
        conflicts.sort_by(|a, b| a.conflict_path.cmp(&b.conflict_path));

        assert!(conflicts[0].conflict_path.contains("sub"));
        assert_eq!(conflicts[0].original_size, Some(12));
        assert!(conflicts[1].conflict_path.contains(conflict_name));
        assert_eq!(conflicts[1].conflict_size, 16);
        assert_eq!(conflicts[1].original_size, Some(16));
    }

    #[tokio::test]
    async fn test_list_conflicts_tool() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let temp = tempfile::tempdir().unwrap();

        // Create conflict file
        let conflict_name = "test.sync-conflict-20230101-120000-ABCDEFG.txt";
        let conflict_path = temp.path().join(conflict_name);
        tokio::fs::write(&conflict_path, "conflict content")
            .await
            .unwrap();

        Mock::given(method("GET"))
            .and(path("/rest/config/folders/default"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "default",
                "path": temp.path().to_string_lossy(),
                "label": "Default Folder",
                "type": "sendreceive",
                "devices": []
            })))
            .mount(&server)
            .await;

        let client = SyncThingClient::new(crate::config::InstanceConfig {
            url: server.uri(),
            api_key: Some("test".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let params = json!({
            "folder_id": "default"
        });

        let result = list_conflicts(client, config, params).await.unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Conflicts in folder default:"));
        assert!(text.contains(conflict_name));
    }
}
