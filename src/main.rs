use syncthing_mcp_rs::config::AppConfig;
use syncthing_mcp_rs::mcp::McpServer;
use syncthing_mcp_rs::tools::create_registry;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting SyncThing MCP server...");

    // 2. Load config
    let config = match AppConfig::load(None, std::env::args().collect())? {
        syncthing_mcp_rs::config::ConfigResult::Config(c) => c,
        syncthing_mcp_rs::config::ConfigResult::Exit => return Ok(()),
    };
    tracing::debug!("Config loaded: {:?}", config);

    // 3. Create tool registry
    let registry = create_registry();

    // 4. Create MCP server
    let (server, rx) = McpServer::new(registry, config);

    // 5. Run server
    tracing::info!("MCP server running on stdio");
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
