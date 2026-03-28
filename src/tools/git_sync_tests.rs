use crate::api::models::Config;
use crate::tools::git_sync::ConfigExporter;
use serde_json::json;

#[test]
fn test_export_config_to_json() {
    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: json!({"enabled": true}),
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
        gui: json!({"enabled": true}),
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
        gui: json!({
            "enabled": true,
            "user": "admin",
            "password": "secret_password",
            "apiKey": "very_secret_api_key"
        }),
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
    client.run_command(&["config", "user.email", "test@example.com"]).await.unwrap();
    client.run_command(&["config", "user.name", "Test User"]).await.unwrap();
    
    // 3. Create a file and commit
    let test_file = repo_path.join("config.json");
    std::fs::write(&test_file, "{}").unwrap();
    
    client.add("config.json").await.expect("Failed to add file");
    let commit_hash = client.commit("Initial commit").await.expect("Failed to commit");
    
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
        gui: json!({"enabled": true, "password": "secret"}),
        ldap: json!({}),
        options: json!({}),
        remote_ignored_devices: json!([]),
        defaults: json!({}),
    };

    let manager = crate::tools::git_sync::GitSyncManager::new(repo_path.clone());
    manager.init().await.expect("Failed to init");
    
    // Configure user for commit
    let client = crate::tools::git_sync::GitClient::new(repo_path.clone());
    client.run_command(&["config", "user.email", "test@example.com"]).await.unwrap();
    client.run_command(&["config", "user.name", "Test User"]).await.unwrap();

    let commit_hash = manager.backup_config(config).await.expect("Failed to backup");
    assert!(!commit_hash.is_empty());
    
    // Verify files exist
    assert!(repo_path.join("config.json").exists());
    assert!(repo_path.join("config.yaml").exists());
    
    // Verify masking in the file
    let json_content = std::fs::read_to_string(repo_path.join("config.json")).unwrap();
    assert!(json_content.contains("\"password\": \"********\""));
}
