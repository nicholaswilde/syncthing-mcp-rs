# :arrows_counterclockwise: Syncthing MCP Server (Rust) :robot:

[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/syncthing-mcp-rs/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/syncthing-mcp-rs?branch=main)
[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/syncthing-mcp-rs/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/syncthing-mcp-rs/actions/workflows/ci.yml)

> [!WARNING]
> This project is currently in active development (v0.1.10) and is **not production-ready**. Features may change, and breaking changes may occur without notice. **Use this MCP server at your own risk.**

A Rust implementation of a [Syncthing](https://syncthing.net/) [MCP (Model Context Protocol) server](https://modelcontextprotocol.io/docs/getting-started/intro). This server connects to one or more Syncthing instances and exposes tools to monitor and manage file synchronization via the Model Context Protocol.

## :sparkles: Features

- **Multi-Transport Support:**
  - **Stdio:** Default transport for local integrations (e.g., Claude Desktop).
  - **HTTP with SSE:** Remote access support via Server-Sent Events (SSE) for notifications and HTTP POST for messages.
- **Multi-Instance Management:** Manage and target multiple Syncthing instances from a single MCP server. Tools accept an optional `instance` argument (name or index).
- **Multi-Instance Synchronization:** Synchronize configuration (folders and devices) from a source instance to a destination instance.
- **Event Notifications:** Receive real-time MCP notifications for key Syncthing events (e.g., folder state changes, device connections).
- **Robust Configuration:** Supports configuration via CLI arguments, environment variables, and configuration files (**TOML**).
- **Security & Privacy:**
  - **OS Keyring Integration:** Securely store and retrieve API keys from the OS-level secret store.
  - **Authenticated Encryption:** Support for encrypted configuration fields using ChaCha20-Poly1305.
- **Authentication:** Connects to Syncthing using API Key (`X-API-Key`). Supports plain text, OS Keyring (`keyring:service:account`), or encrypted blobs (`encrypted:v1:...`).
- **Resilience:** Automatic retry with exponential backoff for transient network and server errors.
- **Advanced Conflict Management:** Metadata-driven conflict detection and resolution with support for semantic diffing (JSON/YAML) and resolution previews.
- **Bandwidth Orchestration:** Dynamic upload/download rate limiting across instances with support for scheduled performance profiles (e.g., "working_hours").
- **Self-Healing Monitor:** Automated detection and resolution of common Syncthing issues, including stuck folders (via rescans) and offline devices (via reconnection retries with exponential backoff).
- **Version Control Integration (Git-Sync):** Automatically back up Syncthing configurations to a Git repository. Supports sensitive information masking, version diffing, and rolling back to previous configurations.
- **Binary Optimization:** Small footprint (approx. 2.4M) for efficient deployment.
  - **Tools:**
    - `analyze_error`: Analyze a technical error message and provide a diagnostic summary with actionable advice.
    - `browse_folder`: Browse the contents of a synced folder, listing files and subdirectories with optional prefix and recursion depth control.
    - `configure_sharing`: Configure folder sharing between devices (share or unshare).
    - `delete_conflict`: Permanently delete a Syncthing conflict file.
    - `diff_conflicts`: Compare the original and conflict versions of a file.
    - `diff_instance_configs`: Returns a detailed difference report between two SyncThing instance configurations.
    - `get_bandwidth_status`: Get current bandwidth limits and active profiles for all SyncThing instances.
    - `get_device_statistics`: Get detailed connection statistics for all devices, including last seen time and last connection duration.
    - `get_folder_statistics`: Get detailed statistics for all folders, including last scan time and information about the last synced file.
    - `get_global_dashboard`: Get a high-level overview of all configured SyncThing instances, including aggregated transfer rates and network health.
    - `get_instance_health`: Get detailed health information for a specific Syncthing instance, including connectivity, version, uptime, and resource usage.
    - `get_sync_status`: Get detailed synchronization status, state, and completion percentage for a specific folder or device.
    - `get_system_connections`: Get the current connection status and data transfer statistics for all connected devices.
    - `get_system_log`: Get recent log entries from the Syncthing service for troubleshooting.
    - `get_system_status`: Get comprehensive system status information, including version, uptime, memory usage, and the unique device ID.
    - `list_conflicts`: List Syncthing conflict files in a specific folder.
    - `list_instances`: List all configured Syncthing instances and their current health status.
    - `maintain_system`: Perform system maintenance: force a rescan of folders, restart the Syncthing service, or shut down the service.
    - `manage_devices`: Manage Syncthing devices: list, add, remove, pause, resume, approve pending devices, or validate device IDs.
    - `manage_folders`: Manage Syncthing folders: list configured folders, get a specific folder, view pending folder requests, reject pending requests, or revert local changes in Receive Only folders.
    - `manage_ignores`: Manage folder ignore patterns (.stignore). Supports getting current patterns, setting a new list, or appending to the existing list.
    - `merge_instance_configs`: Merges configuration from one SyncThing instance into another. This appends/updates folders and devices instead of replacing the entire configuration.
    - `monitor_self_healing`: Monitor tool that checks for stuck folders and disconnected devices, and triggers self-healing actions.
    - `preview_conflict_resolution`: Show what the file will look like after a proposed resolution.
    - `replicate_config`: Replicate configuration (folders and devices) from one Syncthing instance to another. Optionally perform a dry run or select specific folders/devices.
    - `resolve_conflict`: Resolve a Syncthing conflict file by keeping either the original or the conflict version. Supports a preview mode.
    - `set_bandwidth_limits`: Set the bandwidth limits (upload/download) across one or all SyncThing instances.
    - `set_performance_profile`: Set the active performance profile (e.g., 'working_hours', 'overnight', 'full_speed').

## :package: Installation

### Homebrew

> [!NOTE]
> The brew installation method is not currently implemented.

```bash
brew install nicholaswilde/tap/syncthing-mcp-rs
```

## :hammer_and_wrench: Build

To build the project, you need a Rust toolchain installed. For cross-compilation, [cross](https://github.com/cross-rs/cross) is used.

### Local Build

```bash
# Build in release mode
task build:local
```

The binary will be available at `target/release/syncthing-mcp-rs`. The release binary is highly optimized for size (approx. 2.4M).

### Cross-Compilation

Supported architectures can be built using `task`:

```bash
# Build for AMD64 (x86_64)
task build:amd64

# Build for ARM64 (aarch64)
task build:arm64

# Build for ARMv7
task build:armv7

# Build for all supported architectures
task build
```

## :rocket: Usage

### :keyboard: Command Line Interface

The server can be configured via CLI arguments or environment variables.

```bash
# Run the MCP server
./target/release/syncthing-mcp-rs --host "localhost" --port 8384 --api-key "your-api-key"

# Run the MCP server with HTTP/SSE enabled
./target/release/syncthing-mcp-rs --http-enabled --http-port 3000

# Encrypt a sensitive value (e.g., API key) for use in config.toml
./target/release/syncthing-mcp-rs encrypt "your-api-key"
```

### :joystick: HTTP/SSE Remote Access

To access the server remotely, you can enable the HTTP/SSE transport.

1. **Start the server:**
   ```bash
   ./syncthing-mcp-rs --http-enabled --http-port 3000 --http-api-key "your-secret-token"
   ```

2. **Establish an SSE connection:**
   ```bash
   curl -N -H "Authorization: Bearer your-secret-token" http://localhost:3000/sse
   ```
   The first event will contain the endpoint for POSTing messages:
   ```
   event: endpoint
   data: /message?session_id=d623f749-33f6-41e2-91a4-8d440171d8ab
   ```

3. **Send MCP messages via HTTP POST:**
   ```bash
   curl -X POST "http://localhost:3000/message?session_id=d623f749-33f6-41e2-91a4-8d440171d8ab" \
     -H "Authorization: Bearer your-secret-token" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
   ```

#### Available Arguments

| Argument | Environment Variable | Description | Default |
| :--- | :--- | :--- | :--- |
| `-c, --config` | - | Path to configuration file | `config.toml` |
| `--host` | `SYNCTHING_HOST` | Syncthing instance host | `localhost` |
| `--port` | `SYNCTHING_PORT` | Syncthing instance port | `8384` |
| `--api-key` | `SYNCTHING_API_KEY` | Syncthing API key (supports `keyring:...` and `encrypted:...`) | - |
| `--transport` | `SYNCTHING_MCP_TRANSPORT` | Transport mode (`stdio`) | `stdio` |
| `--http-enabled` | `SYNCTHING_HTTP_SERVER__ENABLED` | Enable the HTTP/SSE server | `false` |
| `--http-host` | `SYNCTHING_HTTP_SERVER__HOST` | HTTP server host | `0.0.0.0` |
| `--http-port` | `SYNCTHING_HTTP_SERVER__PORT` | HTTP server port | `3000` |
| `--http-api-key` | `SYNCTHING_HTTP_SERVER__API_KEY` | Bearer token for HTTP server | - |
| `--no-verify-ssl` | `SYNCTHING_NO_VERIFY_SSL` | Disable SSL certificate verification | `true` |
| `--log-level` | `SYNCTHING_LOG_LEVEL` | Log level (`info`, `debug`, etc.) | `info` |
| - | `SYNCTHING_RETRY_MAX_ATTEMPTS` | Max retries for API calls | `3` |
| - | `SYNCTHING_RETRY_INITIAL_BACKOFF_MS` | Initial retry backoff in ms | `100` |
| - | `SYNCTHING_INSTANCES__<N>__<FIELD>` | Configuration for multiple instances | - |

### :file_folder: Configuration File

The server automatically looks for `config.toml` in the current directory and `~/.config/syncthing-mcp-rs/`.

#### Multi-Instance Configuration

```toml
# Global settings
retry_max_attempts = 3
retry_initial_backoff_ms = 100

# Default instance using OS Keyring
host = "localhost"
port = 8384
api_key = "keyring:syncthing:local-key"

# Bandwidth Orchestration (Optional)
[bandwidth]
active_profile = "working_hours"

[[bandwidth.profiles]]
name = "working_hours"
limits = { max_recv_kbps = 1000, max_send_kbps = 500 }

[[bandwidth.profiles]]
name = "full_speed"
limits = { max_recv_kbps = 0, max_send_kbps = 0 }

[[bandwidth.schedules]]
profile_name = "working_hours"
days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
start_time = "09:00"
end_time = "17:00"

# OR use the instances list with encrypted values
[[instances]]
name = "remote"
url = "https://sync.example.com"
api_key = "encrypted:v1:4jBYSgrSQQ0JbZzLwBom99zlcKJ8549tgeSUnm4dZr+L4+1rPh0WFak="
no_verify_ssl = false
```

### :robot: Configuration Example (Claude Desktop)

Add the following to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "syncthing": {
      "command": "/path/to/syncthing-mcp-rs/target/release/syncthing-mcp-rs",
      "args": [
        "--host", "localhost",
        "--port", "8384",
        "--api-key", "your-api-key"
      ]
    }
  }
}
```

## :test_tube: Testing

The project uses [go-task](https://taskfile.dev/) for development tasks.

```bash
# Run all checks (format, lint, unit tests)
task test:ci

# Run unit tests only
task test

# Run Docker integration tests (requires Docker)
RUN_DOCKER_TESTS=true task test:integration

# Run MCP Inspector (requires npx)
task inspector

# Generate documentation
task docs

# Generate and open documentation
task docs:open

# Update cargo dependencies
task update
```

### :bar_chart: Coverage

The project uses `cargo-llvm-cov` for code coverage analysis.

```bash
# Show coverage summary in console
task coverage

# Generate detailed HTML and LCOV reports
task coverage:report

# Upload coverage to Coveralls.io (requires COVERALLS_REPO_TOKEN)
COVERALLS_REPO_TOKEN=your_token task coverage:upload
```

## :handshake: Contributing

Contributions are welcome! Please follow standard Rust coding conventions and ensure all tests pass (`task check`) before submitting features.

## :balance_scale: License

[Apache License 2.0](LICENSE)

## :writing_hand: Author

This project was started in 2026 by [Nicholas Wilde][2].

[2]: <https://github.com/nicholaswilde/>
