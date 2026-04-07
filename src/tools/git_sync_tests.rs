use crate::api::models::Config;
use crate::tools::git_sync::ConfigExporter;
use serde_json::json;

#[test]
fn test_export_config_to_json() {
    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: crate::api::models::GuiConfig {
            enabled: true,
            ..Default::default()
        },
        ldap: json!({}),
        options: json!({"listenAddresses": ["default"]}),
        remote_ignored_devices: json!([]),
        defaults: json!({}),
    };

    let exporter = ConfigExporter::new(config);
    let exported = exporter.to_json().expect("Failed to export to JSON");

    // Verify it's pretty-printed JSON
    assert!(exported.contains("  \"version\": 37"));
    assert!(exported.contains("\"gui\": {"));
}

#[test]
fn test_export_config_to_yaml() {
    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: crate::api::models::GuiConfig {
            enabled: true,
            ..Default::default()
        },
        ldap: json!({}),
        options: json!({"listenAddresses": ["default"]}),
        remote_ignored_devices: json!([]),
        defaults: json!({}),
    };

    let exporter = ConfigExporter::new(config);
    let exported = exporter.to_yaml().expect("Failed to export to YAML");

    // Verify it's YAML
    assert!(exported.contains("version: 37"));
    assert!(exported.contains("gui:"));
    assert!(exported.contains("enabled: true"));
}

#[test]
fn test_mask_sensitive_info() {
    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: crate::api::models::GuiConfig {
            enabled: true,
            user: Some("admin".to_string()),
            password: Some("secret_password".to_string()),
            api_key: Some("very_secret_api_key".to_string()),
            ..Default::default()
        },
        ldap: json!({
            "enabled": true,
            "password": "ldap_password"
        }),
        options: json!({}),
        remote_ignored_devices: json!([]),
        defaults: json!({}),
    };

    let mut exporter = ConfigExporter::new(config);
    exporter.mask_sensitive();
    let exported = exporter.to_json().expect("Failed to export to JSON");

    // Verify sensitive info is masked
    assert!(exported.contains("\"user\": \"********\""));
    assert!(exported.contains("\"password\": \"********\""));
    assert!(exported.contains("\"apiKey\": \"********\""));

    // Verify non-sensitive info is preserved
    assert!(exported.contains("\"enabled\": true"));
}

#[tokio::test]
async fn test_git_client_init_and_commit() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    let client = crate::tools::git_sync::GitClient::new(repo_path.to_path_buf());

    // 1. Initialize repo
    client.init().await.expect("Failed to init git repo");
    assert!(repo_path.join(".git").exists());

    // 2. Configure user for commit
    client
        .run_command(&["config", "user.email", "test@example.com"])
        .await
        .unwrap();
    client
        .run_command(&["config", "user.name", "Test User"])
        .await
        .unwrap();

    // 3. Create a file and commit
    let test_file = repo_path.join("config.json");
    std::fs::write(&test_file, "{}").unwrap();

    client.add("config.json").await.expect("Failed to add file");
    let commit_hash = client
        .commit("Initial commit")
        .await
        .expect("Failed to commit");

    assert!(!commit_hash.is_empty());
}

#[tokio::test]
async fn test_backup_config() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path().to_path_buf();

    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: crate::api::models::GuiConfig {
            enabled: true,
            password: Some("secret".to_string()),
            ..Default::default()
        },
        ldap: json!({}),
        options: json!({}),
        remote_ignored_devices: json!([]),
        defaults: json!({}),
    };

    let manager = crate::tools::git_sync::GitSyncManager::new(repo_path.clone());
    manager.init().await.expect("Failed to init");

    // Configure user for commit
    let client = crate::tools::git_sync::GitClient::new(repo_path.clone());
    client
        .run_command(&["config", "user.email", "test@example.com"])
        .await
        .unwrap();
    client
        .run_command(&["config", "user.name", "Test User"])
        .await
        .unwrap();

    let commit_hash = manager
        .backup_config(config)
        .await
        .expect("Failed to backup");
    assert!(!commit_hash.is_empty());

    // Verify files exist
    assert!(repo_path.join("config.json").exists());
    assert!(repo_path.join("config.yaml").exists());

    // Verify masking in the file
    let json_content = std::fs::read_to_string(repo_path.join("config.json")).unwrap();
    assert!(json_content.contains("\"password\": \"********\""));
}

#[tokio::test]
async fn test_git_client_clone() {
    let temp_source = tempfile::tempdir().expect("Failed to create source dir");
    let source_path = temp_source.path();

    // Init source repo
    let source_client = crate::tools::git_sync::GitClient::new(source_path.to_path_buf());
    source_client.init().await.unwrap();
    std::fs::write(source_path.join("README.md"), "# Test").unwrap();
    source_client.add("README.md").await.unwrap();
    source_client
        .run_command(&["config", "user.email", "test@example.com"])
        .await
        .unwrap();
    source_client
        .run_command(&["config", "user.name", "Test User"])
        .await
        .unwrap();
    source_client.commit("Initial").await.unwrap();

    // Clone to destination
    let temp_dest = tempfile::tempdir().expect("Failed to create dest dir");
    let dest_path = temp_dest.path().join("cloned"); // clone into a subdir

    let dest_client = crate::tools::git_sync::GitClient::new(dest_path.clone());
    dest_client
        .clone_from(&source_path.to_string_lossy())
        .await
        .expect("Failed to clone");

    assert!(dest_path.join(".git").exists());
    assert!(dest_path.join("README.md").exists());
}

#[tokio::test]
async fn test_restore_config() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path().to_path_buf();

    let manager = crate::tools::git_sync::GitSyncManager::new(repo_path.clone());
    manager.init().await.expect("Failed to init");

    // Configure user for commit
    let client = crate::tools::git_sync::GitClient::new(repo_path.clone());
    client
        .run_command(&["config", "user.email", "test@example.com"])
        .await
        .unwrap();
    client
        .run_command(&["config", "user.name", "Test User"])
        .await
        .unwrap();

    // 1. First backup
    let config1 = Config {
        version: 1,
        ..Default::default()
    };
    let hash1 = manager
        .backup_config(config1)
        .await
        .expect("Failed backup 1");

    // 2. Second backup
    let config2 = Config {
        version: 2,
        ..Default::default()
    };
    let _hash2 = manager
        .backup_config(config2)
        .await
        .expect("Failed backup 2");

    // 3. Restore to first backup
    let restored = manager
        .restore_config(&hash1)
        .await
        .expect("Failed to restore");
    assert_eq!(restored.version, 1);
}

#[tokio::test]
async fn test_get_config_diff() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path().to_path_buf();

    let manager = crate::tools::git_sync::GitSyncManager::new(repo_path.clone());
    manager.init().await.expect("Failed to init");

    // Configure user for commit
    let client = crate::tools::git_sync::GitClient::new(repo_path.clone());
    client
        .run_command(&["config", "user.email", "test@example.com"])
        .await
        .unwrap();
    client
        .run_command(&["config", "user.name", "Test User"])
        .await
        .unwrap();

    // 1. First backup
    let config1 = Config {
        version: 1,
        ..Default::default()
    };
    let hash1 = manager
        .backup_config(config1)
        .await
        .expect("Failed backup 1");

    // 2. Second backup
    let config2 = Config {
        version: 2,
        ..Default::default()
    };
    let hash2 = manager
        .backup_config(config2)
        .await
        .expect("Failed backup 2");

    // 3. Get diff
    let diff = manager
        .get_diff(&hash1, &hash2)
        .await
        .expect("Failed to get diff");

    assert!(diff.contains("-  \"version\": 1"));
    assert!(diff.contains("+  \"version\": 2"));
}

#[tokio::test]
async fn test_git_client_clone_error() {
    let temp_dest = tempfile::tempdir().expect("Failed to create dest dir");
    let dest_path = temp_dest.path().to_path_buf();

    let dest_client = crate::tools::git_sync::GitClient::new(dest_path.clone());
    let result = dest_client.clone_from("invalid-url").await;
    assert!(result.is_err());
    assert!(format!("{:?}", result).contains("Git clone failed"));
}

#[tokio::test]
async fn test_git_client_run_command_error() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    let client = crate::tools::git_sync::GitClient::new(repo_path.to_path_buf());
    client.init().await.unwrap();

    let result = client.run_command(&["invalid-command"]).await;
    assert!(result.is_err());
    assert!(format!("{:?}", result).contains("Git command failed"));
}

#[tokio::test]
async fn test_git_sync_manager_init_remote() {
    let temp_source = tempfile::tempdir().expect("Failed to create source dir");
    let source_path = temp_source.path();

    // Init source repo
    let source_client = crate::tools::git_sync::GitClient::new(source_path.to_path_buf());
    source_client.init().await.unwrap();
    std::fs::write(source_path.join("README.md"), "# Test").unwrap();
    source_client.add("README.md").await.unwrap();
    source_client
        .run_command(&["config", "user.email", "test@example.com"])
        .await
        .unwrap();
    source_client
        .run_command(&["config", "user.name", "Test User"])
        .await
        .unwrap();
    source_client.commit("Initial").await.unwrap();

    // Clone to destination via manager
    let temp_dest = tempfile::tempdir().expect("Failed to create dest dir");
    let dest_path = temp_dest.path().join("cloned");

    let manager = crate::tools::git_sync::GitSyncManager::new(dest_path.clone());
    manager
        .init_remote(&source_path.to_string_lossy())
        .await
        .expect("Failed to init remote");

    assert!(dest_path.join(".git").exists());
    assert!(dest_path.join("README.md").exists());
}

#[tokio::test]
async fn test_git_sync_manager_push() {
    let temp_remote = tempfile::tempdir().expect("Failed to create remote dir");
    let remote_path = temp_remote.path();

    // Init bare remote repo
    let remote_client = crate::tools::git_sync::GitClient::new(remote_path.to_path_buf());
    remote_client
        .run_command(&["init", "--bare"])
        .await
        .unwrap();

    // Local repo
    let temp_local = tempfile::tempdir().expect("Failed to create local dir");
    let local_path = temp_local.path().to_path_buf();

    let manager = crate::tools::git_sync::GitSyncManager::new(local_path.clone());
    manager.init().await.unwrap();

    // Configure user
    let client = crate::tools::git_sync::GitClient::new(local_path.clone());
    client
        .run_command(&["config", "user.email", "test@example.com"])
        .await
        .unwrap();
    client
        .run_command(&["config", "user.name", "Test User"])
        .await
        .unwrap();

    // Create a backup
    manager
        .backup_config(Config::default())
        .await
        .expect("Failed backup");

    // Get current branch name
    let branch = client
        .run_command(&["rev-parse", "--abbrev-ref", "HEAD"])
        .await
        .unwrap();
    let branch = branch.trim();

    // Add remote
    client
        .run_command(&["remote", "add", "origin", &remote_path.to_string_lossy()])
        .await
        .unwrap();

    // Push via manager
    manager
        .push("origin", branch)
        .await
        .expect("Failed to push");

    // Verify remote has the commit (using another client to check)
    let check_client = crate::tools::git_sync::GitClient::new(remote_path.to_path_buf());
    let log = check_client.run_command(&["log", branch]).await.unwrap();
    assert!(log.contains("Backup configuration:"));
}
