use serde_json::json;
use syncthing_mcp_rs::api::models::Config;
use syncthing_mcp_rs::tools::git_sync::{GitClient, GitSyncManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let run_live = std::env::var("RUN_LIVE_TESTS").unwrap_or_default();
    if run_live != "1" && run_live != "true" {
        println!("Skipping live test script (RUN_LIVE_TESTS not set to 1 or true)");
        return Ok(());
    }
    // 1. Setup Source Repo (Simulating a remote)
    let temp_source = tempfile::tempdir().expect("Failed to create source dir");
    let source_path = temp_source.path().to_path_buf();

    println!(
        "--- Setting up Source (Remote) Repo at {:?} ---",
        source_path
    );
    let source_client = GitClient::new(source_path.clone());
    source_client.init().await.expect("Failed to init source");
    source_client
        .run_command(&["config", "user.email", "source@example.com"])
        .await
        .unwrap();
    source_client
        .run_command(&["config", "user.name", "Source User"])
        .await
        .unwrap();
    source_client
        .run_command(&["config", "receive.denyCurrentBranch", "ignore"])
        .await
        .unwrap();

    // Create a dummy file to have an initial commit
    std::fs::write(source_path.join("README.md"), "# Backup Repo").unwrap();
    source_client.add("README.md").await.unwrap();
    source_client.commit("Initial commit").await.unwrap();

    // 2. Setup Destination Repo (Cloning from source)
    let temp_dest = tempfile::tempdir().expect("Failed to create dest dir");
    let dest_path = temp_dest.path().join("cloned_backup");

    println!("\n--- Cloning to Destination Repo at {:?} ---", dest_path);
    let manager = GitSyncManager::new(dest_path.clone());
    manager
        .init_remote(&source_path.to_string_lossy())
        .await
        .expect("Failed to clone");

    // Configure user for dest commits
    let dest_client = GitClient::new(dest_path.clone());
    dest_client
        .run_command(&["config", "user.email", "dest@example.com"])
        .await
        .unwrap();
    dest_client
        .run_command(&["config", "user.name", "Dest User"])
        .await
        .unwrap();

    // 3. Perform Backup
    let config = Config {
        version: 37,
        folders: vec![],
        devices: vec![],
        gui: syncthing_mcp_rs::api::models::GuiConfig {
            enabled: true,
            password: Some("secret".to_string()),
            ..Default::default()
        },
        ldap: json!({}),
        options: json!({}),
        remote_ignored_devices: json!([]),
        defaults: json!({}),
    };

    println!("\n--- Performing Configuration Backup in Cloned Repo ---");
    let commit_hash = manager
        .backup_config(config)
        .await
        .expect("Failed to backup");
    println!("Local Commit Hash: {}", commit_hash);

    // 4. Push back to "remote"
    println!("\n--- Pushing Changes back to Source ---");
    manager
        .push("origin", "main")
        .await
        .expect("Failed to push");

    // 5. Verify at Source
    println!("\n--- Verifying Changes at Source ---");
    // We need to reset the working tree at the source because we pushed to a non-bare repo's current branch
    source_client
        .run_command(&["reset", "--hard", "main"])
        .await
        .unwrap();

    let source_log = source_client
        .run_command(&["log", "-1", "--oneline"])
        .await
        .unwrap();
    println!("Latest Source Commit: {}", source_log.trim());

    if source_path.join("config.json").exists() {
        println!("SUCCESS: config.json synchronized to source repo.");
    } else {
        println!("FAILURE: config.json missing from source repo.");
    }

    Ok(())
}
