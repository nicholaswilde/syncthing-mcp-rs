# Initial Concept

An mcp server to control SyncThing

# Product Definition - SyncThing MCP Server (Rust)

## Vision
A high-performance, secure Model Context Protocol (MCP) server written in Rust that provides a seamless interface for Large Language Models (LLMs) to manage and monitor SyncThing instances. This server acts as a sophisticated proxy for the SyncThing REST API, enabling autonomous folder management, device synchronization, and system monitoring.

## Core Goals
1. **Unified Management**: Control multiple SyncThing instances through a single MCP interface using an "instances" configuration pattern.
2. **Context-Efficient Tools**: Provide high-level, functional tools (e.g., `manage_folders`, `manage_devices`) rather than exposing raw, granular API endpoints, optimizing for LLM token usage.
3. **Protocol Versatility**: Support both `stdio` transport for local integration (e.g., Claude Desktop) and `HTTP/SSE` for remote management.
4. **Reliability & Performance**: Leverage Rust's safety and performance to ensure stable, low-latency interactions with the SyncThing API.
5. **End-to-End Verification**: Utilize automated integration tests with real SyncThing Docker instances to guarantee tool correctness and API compatibility.
6. **Security First**: Implement secure credential handling (supporting API Keys), OS Keyring integration, and authenticated encryption (ChaCha20-Poly1305) for configuration fields.

## Target Users
- **Developers & Power Users**: Who want to automate their file synchronization workflows using LLMs.
- **System Administrators**: Managing clusters of SyncThing instances across different environments.
- **AI Enthusiasts**: Building autonomous agents that require access to local or remote file synchronization states.

## Key Features (MVP)
- **Folder Management**: List, create, modify, and delete SyncThing folders; monitor synchronization status.
- **File Browsing**: Explore files and subdirectories within synced folders.
- **Device Management**: Add, remove, and configure SyncThing devices; track connection states.
- **Device Discovery**: Discover and approve new device connection requests.
- **System Monitoring**: Access real-time statistics, logs, and health checks for managed instances.
- **Event Notifications**: Receive real-time MCP notifications for key SyncThing events (e.g., folder state changes).
- **System Maintenance**: Perform critical operations such as triggering rescans, clearing errors, and restarting SyncThing instances.
- **Instance Synchronization**: Ability to replicate configuration (folders and devices) across multiple instances from a source instance.
- **Flexible Configuration**: Support for TOML configuration files and environment variables (optimized for binary size).
