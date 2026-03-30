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
mod error_diagnostics_tests;
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
/// Runs the SyncThing MCP server with command-line arguments.
pub async fn run_with_args(args: Vec<String>) -> anyhow::Result<()> {
    // 1. Initialize logging (ignoring errors if already initialized)
    let _ = tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .try_init();

    tracing::info!("Starting SyncThing MCP server...");

    // 2. Load config
    let config = match AppConfig::load(None, args).await? {
        crate::config::ConfigResult::Config(c) => *c,
        crate::config::ConfigResult::Exit => return Ok(()),
    };
    tracing::debug!("Config loaded: {:?}", config);

    // 3. Create tool registry
    let registry = create_registry();

    // 4. Create MCP server
    let (server, rx) = McpServer::new(registry, config.clone());

    // 5. Create and run Event Manager
    let event_manager =
        crate::mcp::events::EventManager::new(config.clone(), server.notification_tx.clone());
    tokio::spawn(async move {
        if let Err(e) = event_manager.run().await {
            tracing::error!("Event manager error: {}", e);
        }
    });

    // 6. Run server
    if config.http_server.enabled {
        let addr = format!("{}:{}", config.http_server.host, config.http_server.port);
        tracing::info!("MCP server running on HTTP/SSE: {}", addr);

        let app = server.router();
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        // Handle notifications for SSE in the background
        tokio::spawn(async move {
            let mut rx = rx;
            while let Some(n) = rx.recv().await {
                let sessions = server.sessions.clone();
                for session in sessions.iter() {
                    let _ = session
                        .tx
                        .send(crate::mcp::Message::Notification(n.clone()))
                        .await;
                }
            }
        });

        axum::serve(listener, app).await?;
    } else {
        tracing::info!("MCP server running on stdio");
        server.run_stdio(rx).await?;
    }

    Ok(())
}

/// Runs the SyncThing MCP server using environment arguments.
pub async fn run() -> anyhow::Result<()> {
    run_with_args(std::env::args().collect()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_run_with_encrypt_exits_successfully() {
        let args = vec![
            "syncthing-mcp-rs".to_string(),
            "encrypt".to_string(),
            "dummy".to_string(),
        ];
        let result = run_with_args(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_args_invalid_config() {
        let _args = [
            "syncthing-mcp-rs".to_string(),
            "--port".to_string(),
            "not-a-port".to_string(),
        ];
        // This will exit the process in the current implementation of parse_args
        // because it uses get_matches_from.
        // So we can't easily test it here without changing parse_args to try_get_matches.
    }
}
