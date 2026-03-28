#[cfg(test)]
mod tests {
    use crate::tools::diff::{DiffFormat, get_diff, get_text_diff};

    #[test]
    fn test_get_text_diff() {
        let original = "line 1\nline 2\nline 3";
        let conflict = "line 1\nline 2 modified\nline 3\nline 4";
        let diff = get_text_diff(original, conflict);
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+line 2 modified"));
        assert!(diff.contains("+line 4"));
    }

    #[test]
    fn test_get_diff_auto_text() {
        let original = "line 1\nline 2";
        let conflict = "line 1\nline 2 mod";
        let diff = get_diff(original, conflict, DiffFormat::Auto).unwrap();
        println!("Diff: {}", diff);
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+line 2 mod"));
    }

    #[test]
    fn test_get_json_diff() {
        let original = r#"{"a": 1, "b": 2}"#;
        let conflict = r#"{"a": 1, "b": 3, "c": 4}"#;
        let diff = get_diff(original, conflict, DiffFormat::Json).unwrap();
        assert!(diff.contains("\"b\""));
        assert!(diff.contains("\"c\""));
    }

    #[test]
    fn test_get_yaml_diff() {
        let original = "a: 1\nb: 2";
        let conflict = "a: 1\nb: 3\nc: 4";
        let diff = get_diff(original, conflict, DiffFormat::Yaml).unwrap();
        assert!(diff.contains("b"));
        assert!(diff.contains("c"));
    }

    #[test]
    fn test_get_resolution_preview_keep_original() {
        let original = "original content";
        let conflict = "conflict content";
        let preview =
            crate::tools::diff::get_resolution_preview(original, conflict, "keep_original");
        assert_eq!(preview, original);
    }

    #[test]
    fn test_get_resolution_preview_keep_conflict() {
        let original = "original content";
        let conflict = "conflict content";
        let preview =
            crate::tools::diff::get_resolution_preview(original, conflict, "keep_conflict");
        assert_eq!(preview, conflict);
    }

    #[test]
    fn test_get_diff_auto_json() {
        let original = r#"{"a": 1}"#;
        let conflict = r#"{"a": 2}"#;
        let diff = get_diff(original, conflict, DiffFormat::Auto).unwrap();
        assert!(diff.contains("\"a\""));
    }

    #[test]
    fn test_get_diff_auto_yaml() {
        let original = "a: 1";
        let conflict = "a: 2";
        let diff = get_diff(original, conflict, DiffFormat::Auto).unwrap();
        assert!(diff.contains("a"));
    }

    #[test]
    fn test_get_json_diff_no_changes() {
        let original = r#"{"a": 1}"#;
        let conflict = r#"{"a": 1}"#;
        let diff = get_diff(original, conflict, DiffFormat::Json).unwrap();
        assert_eq!(diff, "No changes detected in JSON structure.");
    }

    #[test]
    fn test_get_yaml_diff_no_changes() {
        let original = "a: 1";
        let conflict = "a: 1";
        let diff = get_diff(original, conflict, DiffFormat::Yaml).unwrap();
        assert_eq!(diff, "No changes detected in YAML structure.");
    }

    #[test]
    fn test_get_json_diff_invalid() {
        let original = r#"{"a": 1}"#;
        let conflict = r#"{"a": 1"#; // Invalid JSON
        let result = get_diff(original, conflict, DiffFormat::Json);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_diff_conflicts_tool() {
        use crate::api::SyncThingClient;
        use crate::config::{AppConfig, InstanceConfig};
        use serde_json::json;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let original_path = dir.path().join("test.txt");
        let conflict_path = dir
            .path()
            .join("test.sync-conflict-20230101-120000-DEVICE.txt");

        std::fs::write(&original_path, "original content").unwrap();
        std::fs::write(&conflict_path, "conflict content").unwrap();

        let client = SyncThingClient::new(InstanceConfig {
            name: Some("test".to_string()),
            url: "http://localhost:8384".to_string(),
            api_key: Some("api-key".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();
        let args = json!({
            "conflict_path": conflict_path.to_str().unwrap()
        });

        let result = crate::tools::diff::diff_conflicts(client, config, args)
            .await
            .unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("-original content"));
        assert!(text.contains("+conflict content"));
    }

    #[tokio::test]
    async fn test_preview_conflict_resolution_tool() {
        use crate::api::SyncThingClient;
        use crate::config::{AppConfig, InstanceConfig};
        use serde_json::json;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let original_path = dir.path().join("test.txt");
        let conflict_path = dir
            .path()
            .join("test.sync-conflict-20230101-120000-DEVICE.txt");

        std::fs::write(&original_path, "original content").unwrap();
        std::fs::write(&conflict_path, "conflict content").unwrap();

        let client = SyncThingClient::new(InstanceConfig {
            name: Some("test".to_string()),
            url: "http://localhost:8384".to_string(),
            api_key: Some("api-key".to_string()),
            ..Default::default()
        });
        let config = AppConfig::default();

        // Test keep_original
        let args = json!({
            "conflict_path": conflict_path.to_str().unwrap(),
            "action": "keep_original"
        });
        let result =
            crate::tools::diff::preview_conflict_resolution(client.clone(), config.clone(), args)
                .await
                .unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "original content");

        // Test keep_conflict
        let args = json!({
            "conflict_path": conflict_path.to_str().unwrap(),
            "action": "keep_conflict"
        });
        let result = crate::tools::diff::preview_conflict_resolution(client, config, args)
            .await
            .unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "conflict content");
    }
}
