use syncthing_mcp_rs::api::models::Config;
use syncthing_mcp_rs::tools::git_sync::{GitClient, GitSyncManager};

#[tokio::test]
async fn test_e2e_backup_and_rollback() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path().to_path_buf();

    let manager = GitSyncManager::new(repo_path.clone());
    manager.init().await.expect("Failed to init");

    // Configure user for commit
    let client = GitClient::new(repo_path.clone());
    client
        .run_command(&["config", "user.email", "e2e@example.com"])
        .await
        .unwrap();
    client
        .run_command(&["config", "user.name", "E2E Test"])
        .await
        .unwrap();

    // 1. Initial State (v1)
    let config1 = Config {
        version: 1,
        gui: syncthing_mcp_rs::api::models::GuiConfig {
            enabled: true,
            user: Some("admin".to_string()),
            password: Some("p1".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let hash1 = manager
        .backup_config(config1)
        .await
        .expect("Failed backup 1");

    // 2. Updated State (v2)
    let config2 = Config {
        version: 2,
        gui: syncthing_mcp_rs::api::models::GuiConfig {
            enabled: true,
            user: Some("admin".to_string()),
            password: Some("p2".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let hash2 = manager
        .backup_config(config2)
        .await
        .expect("Failed backup 2");

    // 3. Verify Diffs
    let diff = manager
        .get_diff(&hash1, &hash2)
        .await
        .expect("Failed to get diff");
    assert!(diff.contains("-  \"version\": 1"));
    assert!(diff.contains("+  \"version\": 2"));

    // 4. Verify Masking in both versions
    let restored1 = manager.restore_config(&hash1).await.unwrap();
    let restored2 = manager.restore_config(&hash2).await.unwrap();

    assert_eq!(restored1.gui.password.as_deref().unwrap(), "********");
    assert_eq!(restored2.gui.password.as_deref().unwrap(), "********");

    // 5. Rollback to v1
    let rolled_back = manager
        .restore_config(&hash1)
        .await
        .expect("Failed rollback");
    assert_eq!(rolled_back.version, 1);
}
