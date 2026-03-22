# Product Guidelines - SyncThing MCP Server

## 1. Code Style & Architecture (Rust)
- **Safety First**: Utilize Rust's ownership and borrowing systems to prevent memory leaks and data races.
- **Async Efficiency**: Use `tokio` for high-concurrency, asynchronous operations, particularly for network-bound tasks with the SyncThing API.
- **Error Handling**: Implement custom error types using crates like `thiserror` or `anyhow` for clear, actionable error messages.
- **Modularity**: Organize code into clean abstractions (e.g., `api/`, `mcp/`, `config/`) to ensure long-term maintainability.
- **Documentation**: Provide inline documentation (using `///` comments) and clear `README` instructions for configuration and setup.

## 2. Model Context Protocol (MCP) Standards
- **High-Signal Tools**: Each tool should provide a concise, high-signal output. Avoid returning massive JSON blobs unless specifically requested.
- **Clear Tool Descriptions**: Use descriptive tool names and comprehensive "tool definition" descriptions to help LLMs understand when and how to use each tool.
- **Parameter Validation**: Rigorously validate all tool inputs before attempting any external API calls.
- **Transparent Logging**: Provide detailed, levels-based logging (using `tracing` or `log`) for troubleshooting MCP/SyncThing interactions.

## 3. Security & Privacy
- **Secure Secret Management**: Never log API keys, passwords, or other sensitive information. Support for SOPS-encrypted configuration is encouraged.
- **Strict Permission Checks**: (If applicable) Ensure the MCP server only performs operations that are explicitly authorized via the provided API credentials.
- **Local-First Privacy**: Prioritize `stdio` transport to keep data flow entirely within the user's local machine unless remote `SSE` is explicitly required.

## 4. User Experience (UX) for CLI & Logs
- **Actionable Logs**: Logs should clearly indicate when the MCP server is connecting, failing, or successfully executing a command on a SyncThing instance.
- **Informative Stats**: Monitoring tools should return a human-readable summary of folder and device states, not just raw API responses.
- **Clean Configuration**: Use a single, well-structured configuration file (e.g., `config.toml`) that is easy to understand and modify.
