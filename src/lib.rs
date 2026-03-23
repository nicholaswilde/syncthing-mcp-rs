pub mod api;
pub mod config;
pub mod credentials;
pub mod error;
#[cfg(test)]
mod error_tests;
pub mod mcp;
#[cfg(test)]
pub mod test_utils;
pub mod tools;

use crate::config::AppConfig;
use crate::mcp::McpServer;
use crate::tools::create_registry;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub use error::{Error, Result};

/// Version of the package.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Runs the SyncThing MCP server.
pub async fn run() -> anyhow::Result<()> {
    // 1. Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting SyncThing MCP server...");

    // 2. Load config
    let config = match AppConfig::load(None, std::env::args().collect())? {
        crate::config::ConfigResult::Config(c) => c,
        crate::config::ConfigResult::Exit => return Ok(()),
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
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.1.0");
    }

    #[tokio::test]
    async fn test_run_compiles() {
        // We can't easily run the server in a test that expects stdio,
        // but we can at least ensure the function signature and imports are correct.
        // If we want to truly test it, we'd need to mock stdio.
    }
}
