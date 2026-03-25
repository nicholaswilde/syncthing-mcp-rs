#[cfg(test)]
mod tests {
    use crate::api::models::{Config, FolderConfig, DeviceConfig};
    use crate::tools::config_diff::{calculate_diff, ConfigDiff};
    use serde_json::json;

    fn mock_config() -> Config {
        Config {
            version: 1,
            folders: vec![],
            devices: vec![],
            gui: json!({}),
            ldap: json!({}),
            options: json!({}),
            remote_ignored_devices: json!([]),
            defaults: json!({}),
        }
    }

    #[test]
    fn test_calculate_diff_no_changes() {
        let base = mock_config();
        let head = mock_config();
        let diff = calculate_diff(&base, &head);
        assert_eq!(diff.folders_added.len(), 0);
        assert_eq!(diff.folders_removed.len(), 0);
        assert_eq!(diff.folders_updated.len(), 0);
    }

    #[test]
    fn test_calculate_diff_folder_added() {
        let base = mock_config();
        let mut head = mock_config();
        head.folders.push(FolderConfig {
            id: "f1".to_string(),
            label: "Folder 1".to_string(),
            path: "/path/1".to_string(),
            folder_type: "sendreceive".to_string(),
            devices: vec![],
            rescan_interval_s: 3600,
            fs_watcher_enabled: true,
            paused: false,
        });

        let diff = calculate_diff(&base, &head);
        assert_eq!(diff.folders_added.len(), 1);
        assert_eq!(diff.folders_added[0].id, "f1");
    }

    #[test]
    fn test_calculate_diff_folder_removed() {
        let mut base = mock_config();
        base.folders.push(FolderConfig {
            id: "f1".to_string(),
            label: "Folder 1".to_string(),
            path: "/path/1".to_string(),
            folder_type: "sendreceive".to_string(),
            devices: vec![],
            rescan_interval_s: 3600,
            fs_watcher_enabled: true,
            paused: false,
        });
        let head = mock_config();

        let diff = calculate_diff(&base, &head);
        assert_eq!(diff.folders_removed.len(), 1);
        assert_eq!(diff.folders_removed[0], "f1");
    }

    #[test]
    fn test_calculate_diff_folder_updated() {
        let mut base = mock_config();
        base.folders.push(FolderConfig {
            id: "f1".to_string(),
            label: "Folder 1".to_string(),
            path: "/path/1".to_string(),
            folder_type: "sendreceive".to_string(),
            devices: vec![],
            rescan_interval_s: 3600,
            fs_watcher_enabled: true,
            paused: false,
        });
        let mut head = mock_config();
        head.folders.push(FolderConfig {
            id: "f1".to_string(),
            label: "Folder 1 Updated".to_string(),
            path: "/path/1".to_string(),
            folder_type: "sendreceive".to_string(),
            devices: vec![],
            rescan_interval_s: 3600,
            fs_watcher_enabled: true,
            paused: false,
        });

        let diff = calculate_diff(&base, &head);
        assert_eq!(diff.folders_updated.len(), 1);
        assert_eq!(diff.folders_updated[0].label, "Folder 1 Updated");
    }

    #[test]
    fn test_apply_patch() {
        use crate::tools::config_diff::{apply_patch, ConfigPatch};

        let mut config = mock_config();
        config.folders.push(FolderConfig {
            id: "f1".to_string(),
            label: "Folder 1".to_string(),
            path: "/path/1".to_string(),
            folder_type: "sendreceive".to_string(),
            devices: vec![],
            rescan_interval_s: 3600,
            fs_watcher_enabled: true,
            paused: false,
        });

        let patch = ConfigPatch {
            folders: vec![
                FolderConfig {
                    id: "f2".to_string(),
                    label: "Folder 2".to_string(),
                    path: "/path/2".to_string(),
                    folder_type: "sendreceive".to_string(),
                    devices: vec![],
                    rescan_interval_s: 3600,
                    fs_watcher_enabled: true,
                    paused: false,
                }
            ],
            folders_to_remove: vec!["f1".to_string()],
            ..Default::default()
        };

        apply_patch(&mut config, &patch).unwrap();

        assert_eq!(config.folders.len(), 1);
        assert_eq!(config.folders[0].id, "f2");
    }
}
