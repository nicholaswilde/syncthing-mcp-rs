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
- **Folder Management**: List configured folders with status; share/unshare folders with devices; view and reject pending folder requests; revert local changes in Receive Only folders.
- **File Browsing**: Browse files and subdirectories within synced folders with prefix and depth control.
- **Real-time Notifications**: Background polling of SyncThing events with real-time push notifications to MCP clients (Folder changes, Device connections, etc.).
- **Device Management**: Manage devices (list, add, remove, pause, resume), approve pending requests, and validate/format device IDs.
- **System Monitoring**: Access comprehensive system status, version information, real-time connection monitoring with transfer stats, system logs for troubleshooting, and high-level instance health overviews.
- **Sync Status**: Query detailed synchronization status, completion percentage, and granular statistics (last seen, last scan, last synced file) for folders and devices.
- **Manage Ignores**: View and modify SyncThing ignore patterns (.stignore).
- **Error Diagnostics**: Analyze and diagnose common SyncThing errors, providing actionable advice.
- **System Maintenance**: Trigger rescans, restart or shut down SyncThing, and clear system errors.
- **Instance Configuration Management**: Generate detailed difference reports between instances; perform additive merges of folders and devices from a source to a target; synchronize configurations with granular control and safety previews.
- **Conflict Management**: Identify, resolve, and clean up SyncThing conflict files with metadata-driven decision support, safe deletion (trash), and advanced intelligence including semantic diffing (JSON/YAML) and resolution previews.
- **Self-Healing Monitor**: Automatically detect and resolve common SyncThing issues, such as stuck folders (via rescans) and offline devices (via reconnection retries with exponential backoff).
- Bandwidth Orchestration: Dynamically manage upload and download rate limits across multiple SyncThing instances, with support for scheduled performance profiles (e.g., 'working_hours').
- Security & Secrets: Secure credential handling with OS Keyring and authenticated encryption.
- Version Control Integration: Automatically back up SyncThing configurations to Git with support for diffing and rollbacks to previous versions.

