mod common;

use common::SyncThingContainer;
use anyhow::Result;

#[tokio::test]
async fn test_container_starts() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = SyncThingContainer::new().await?;
    let status = container.client().get_system_status().await?;
    
    assert!(!status.my_id.is_empty());
    
    Ok(())
}
