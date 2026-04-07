use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use syncthing_mcp_rs::api::SyncThingClient;
use syncthing_mcp_rs::config::AppConfig;
use syncthing_mcp_rs::tools::create_registry;

fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if !p.starts_with("~") {
        return p.to_path_buf();
    }
    if let Ok(home) = env::var("HOME") {
        if p == Path::new("~") {
            return PathBuf::from(home);
        } else {
            return PathBuf::from(home).join(p.strip_prefix("~").unwrap());
        }
    }
    p.to_path_buf()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note: We assume the caller has sourced the .env file or is using `task` which does it.
    let api_key = env::var("SYNCTHING_API_KEY").expect("SYNCTHING_API_KEY must be set");
    let host = env::var("SYNCTHING_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("SYNCTHING_PORT")
        .unwrap_or_else(|_| "8384".to_string())
        .parse::<u16>()?;

    let mut app_config = AppConfig {
        host: host.clone(),
        port,
        api_key: Some(api_key.clone()),
        ..Default::default()
    };
    app_config.validate().await?;

    let client = SyncThingClient::new(app_config.instances[0].clone());
    let registry = create_registry();

    // 1. Find a folder to use for testing
    let folders = client.list_folders().await?;
    let mut folder_to_use = None;

    for folder in folders {
        let expanded = expand_tilde(&folder.path);
        if expanded.exists() {
            println!(
                "Using folder: {} ({}) at {:?}",
                folder.label, folder.id, expanded
            );
            folder_to_use = Some((folder, expanded));
            break;
        } else {
            println!(
                "Skipping folder {} (path {:?} does not exist locally)",
                folder.id, expanded
            );
        }
    }

    let (_test_folder, folder_path) = match folder_to_use {
        Some(f) => f,
        None => {
            println!("Error: No folders found with a valid local path. Skipping file-based tests.");
            return Ok(());
        }
    };

    // 2. Create temporary conflict files (JSON)
    let original_path = folder_path.join("mcp_live_test.json");
    let conflict_path = folder_path.join("mcp_live_test.sync-conflict-20260324-120000-LIVE.json");

    println!(
        "Creating temporary test files (JSON) at {:?} and {:?}...",
        original_path, conflict_path
    );
    fs::write(&original_path, r#"{"a": 1, "b": 2, "nested": {"x": true}}"#)?;
    fs::write(
        &conflict_path,
        r#"{"a": 1, "b": 3, "nested": {"x": false}, "c": 4}"#,
    )?;

    // 3. Test diff_conflicts (JSON)
    println!("\n--- Testing diff_conflicts (JSON) ---");
    let tool = registry
        .get_tool("diff_conflicts")
        .expect("Tool diff_conflicts not found");
    let result = (tool.handler)(
        client.clone(),
        app_config.clone(),
        Some(json!({
            "conflict_path": conflict_path.to_str().unwrap(),
            "format": "json"
        })),
    )
    .await?;
    println!(
        "JSON Diff Result:\n{}",
        result["content"][0]["text"].as_str().unwrap()
    );

    // 4. Test preview_conflict_resolution
    println!("\n--- Testing preview_conflict_resolution (keep_conflict) ---");
    let tool = registry
        .get_tool("preview_conflict_resolution")
        .expect("Tool preview_conflict_resolution not found");
    let result = (tool.handler)(
        client.clone(),
        app_config.clone(),
        Some(json!({
            "conflict_path": conflict_path.to_str().unwrap(),
            "action": "keep_conflict"
        })),
    )
    .await?;
    println!(
        "Preview Result:\n{}",
        result["content"][0]["text"].as_str().unwrap()
    );

    // 5. Test resolve_conflict with preview
    println!("\n--- Testing resolve_conflict with preview: true ---");
    let tool = registry
        .get_tool("resolve_conflict")
        .expect("Tool resolve_conflict not found");
    let result = (tool.handler)(
        client.clone(),
        app_config.clone(),
        Some(json!({
            "conflict_path": conflict_path.to_str().unwrap(),
            "action": "keep_original",
            "preview": true
        })),
    )
    .await?;
    println!(
        "Resolve Preview Result:\n{}",
        result["content"][0]["text"].as_str().unwrap()
    );

    // 6. Cleanup
    println!("\nCleaning up temporary files...");
    fs::remove_file(&original_path)?;
    fs::remove_file(&conflict_path)?;
    println!("Done.");

    Ok(())
}
