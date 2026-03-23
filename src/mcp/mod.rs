#[cfg(test)]
mod error_integration_tests;
#[cfg(test)]
mod http_tests;
#[cfg(test)]
mod event_tests;
/// The MCP server implementation.
pub mod server;
/// Unit tests for the MCP server.
#[cfg(test)]
pub mod tests;
/// JSON-RPC types for the Model Context Protocol.
pub mod types;
/// Event management for SyncThing.
pub mod events;

pub use server::McpServer;
pub use types::*;
