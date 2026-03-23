//! SyncThing MCP Server.
//!
//! This crate provides a Model Context Protocol (MCP) server for interacting with SyncThing.
//! It allows for managing folders, devices, and system status through an MCP-compatible client.

#![deny(missing_docs)]

/// API client and models for SyncThing.
pub mod api;
/// Configuration for the application and SyncThing instances.
pub mod config;
/// Credential management for SyncThing API keys.
pub mod credentials;
/// Error handling for the application.
pub mod error;
#[cfg(test)]
mod error_tests;
/// MCP server implementation and types.
pub mod mcp;
/// Utility functions for testing.
#[cfg(test)]
pub mod test_utils;
/// Tool definitions and handlers for the MCP server.
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
///
/// This function initializes logging, loads the configuration, creates the tool registry,
/// and starts the MCP server on standard input/output.
///
/// # Errors
///
/// Returns an error if configuration loading or server execution fails.
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
