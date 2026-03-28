use syncthing_mcp_rs::api::models::Config;
use syncthing_mcp_rs::tools::git_sync::GitSyncManager;

#[tokio::main]
async fn main() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path().to_path_buf();

    println!("--- Initializing Git Repo at {:?} ---", repo_path);
    let manager = GitSyncManager::new(repo_path.clone());
    manager.init().await.expect("Failed to init");

    // Configure user for commit
    let client = syncthing_mcp_rs::tools::git_sync::GitClient::new(repo_path.clone());
    client
        .run_command(&["config", "user.email", "test@example.com"])
        .await
        .unwrap();
    client
        .run_command(&["config", "user.name", "Test User"])
        .await
        .unwrap();

    // 1. First backup (v1)
    println!("\n--- Performing First Backup (Version 1) ---");
    let config1 = Config {
        version: 1,
        ..Default::default()
    };
    let hash1 = manager
        .backup_config(config1)
        .await
        .expect("Failed backup 1");
    println!("Commit Hash 1: {}", hash1);

    // 2. Second backup (v2)
    println!("\n--- Performing Second Backup (Version 2) ---");
    let config2 = Config {
        version: 2,
        ..Default::default()
    };
    let hash2 = manager
        .backup_config(config2)
        .await
        .expect("Failed backup 2");
    println!("Commit Hash 2: {}", hash2);

    // 3. View Diff
    println!("\n--- Viewing Diff between V1 and V2 ---");
    let diff = manager
        .get_diff(&hash1, &hash2)
        .await
        .expect("Failed to get diff");
    println!("{}", diff);

    // 4. Restore to V1
    println!("\n--- Restoring to Version 1 ({}) ---", hash1);
    let restored = manager
        .restore_config(&hash1)
        .await
        .expect("Failed to restore");
    println!("Restored Configuration Version: {}", restored.version);

    if restored.version == 1 {
        println!("\nSUCCESS: Configuration correctly restored to Version 1.");
    } else {
        println!(
            "\nFAILURE: Restored configuration has unexpected version {}.",
            restored.version
        );
    }
}
