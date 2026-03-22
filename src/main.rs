use syncthing_mcp_rs::config::AppConfig;
use syncthing_mcp_rs::mcp::McpServer;
use syncthing_mcp_rs::tools::create_registry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load config
    let config = AppConfig::load(None, std::env::args().collect())?;

    // 2. Create tool registry
    let registry = create_registry();

    // 3. Create MCP server
    let (server, rx) = McpServer::new(registry, config);

    // 4. Run server
    server.run_stdio(rx).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        // main() is now async and runs the server, difficult to test directly
    }
}
