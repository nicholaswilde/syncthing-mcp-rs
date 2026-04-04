#[cfg(test)]
mod conflict_tests;
#[cfg(test)]
mod error_integration_tests;
#[cfg(test)]
mod event_tests;
/// Event management for SyncThing.
pub mod events;
#[cfg(test)]
mod http_tests;
/// Token usage optimization utilities.
pub mod optimization;
#[cfg(test)]
mod optimization_tests;
/// The MCP server implementation.
pub mod server;
/// Unit tests for the MCP server.
#[cfg(test)]
pub mod tests;
/// JSON-RPC types for the Model Context Protocol.
pub mod types;
/// Unit tests for MCP types.
#[cfg(test)]
mod types_tests;

pub use server::McpServer;
pub use types::*;
