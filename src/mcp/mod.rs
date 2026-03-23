#[cfg(test)]
mod error_integration_tests;
/// The MCP server implementation.
pub mod server;
/// Unit tests for the MCP server.
#[cfg(test)]
pub mod tests;
/// JSON-RPC types for the Model Context Protocol.
pub mod types;

pub use server::McpServer;
pub use types::*;
