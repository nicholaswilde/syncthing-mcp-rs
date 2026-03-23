# Track Specification: HTTP/SSE Transport Support (http_sse_transport_20260322)

## Overview
Implement an axum-based server to expose the Model Context Protocol (MCP) over HTTP with Server-Sent Events (SSE). This enables remote access to the SyncThing MCP server beyond the default stdio transport.

## Scope
- Integrate the `axum` web framework.
- Implement the MCP HTTP/SSE transport layer.
- Support long-lived SSE connections for server-to-client notifications.
- Ensure compatibility with existing MCP server logic.
- Add configuration options for host, port, and security (e.g., API keys).

## Success Criteria
- [ ] Axum server starts and listens on the configured address.
- [ ] Clients can connect and establish an SSE stream.
- [ ] MCP requests are successfully processed via HTTP POST.
- [ ] Notifications from the server are correctly pushed through the SSE stream.
- [ ] Integration tests verify remote tool execution.
