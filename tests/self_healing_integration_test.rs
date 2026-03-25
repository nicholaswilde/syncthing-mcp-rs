mod common;

use anyhow::Result;
use common::TestContext;
use serde_json::json;

#[tokio::test]
async fn test_monitor_self_healing_tool() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let ctx = TestContext::new().await?;
    
    // 1. Call with dry_run
    let result = ctx.call_tool("monitor_self_healing", json!({"dry_run": true})).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Self-Healing Monitor Report:"));
    assert!(text.contains("No actions needed at this time.") || text.contains("Actions Taken:"));

    // 2. Call normally
    let result = ctx.call_tool("monitor_self_healing", json!({})).await?;
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Self-Healing Monitor Report:"));

    Ok(())
}
