# :arrows_counterclockwise: SyncThing MCP Server (Rust) :robot:

[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/syncthing-mcp-rs/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/syncthing-mcp-rs?branch=main)
[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/syncthing-mcp-rs/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/syncthing-mcp-rs/actions/workflows/ci.yml)

> [!WARNING]
> This project is currently in active development (v0.1.0) and is **not production-ready**. Features may change, and breaking changes may occur without notice. **Use this MCP server at your own risk.**

A Rust implementation of a SyncThing [MCP (Model Context Protocol) server](https://modelcontextprotocol.io/docs/getting-started/intro). This server connects to one or more SyncThing instances and exposes tools to monitor and manage file synchronization via the Model Context Protocol.

## :sparkles: Features

- **Multi-Transport Support:**
  - **Stdio:** Default transport for local integrations (e.g., Claude Desktop).
  - **HTTP with SSE:** Remote access support via Server-Sent Events (SSE) for notifications and HTTP POST for messages.
- **Multi-Instance Management:** Manage and target multiple SyncThing instances from a single MCP server. Tools accept an optional `instance` argument (name or index).
- **Multi-Instance Synchronization:** Synchronize configuration (folders and devices) from a source instance to a destination instance.
- **Event Notifications:** Receive real-time MCP notifications for key SyncThing events (e.g., folder state changes, device connections).
- **Robust Configuration:** Supports configuration via CLI arguments, environment variables, and configuration files (**TOML**).
- **Security & Privacy:**
  - **OS Keyring Integration:** Securely store and retrieve API keys from the OS-level secret store.
  - **Authenticated Encryption:** Support for encrypted configuration fields using ChaCha20-Poly1305.
- **Authentication:** Connects to SyncThing using API Key (`X-API-Key`). Supports plain text, OS Keyring (`keyring:service:account`), or encrypted blobs (`encrypted:v1:...`).
- **Resilience:** Automatic retry with exponential backoff for transient network and server errors.
- **Binary Optimization:** Small footprint (approx. 2.4M) for efficient deployment.
  - **Tools:**
    - `list_instances`: List all configured SyncThing instances and their current health status.
    - `get_instance_health`: Get detailed health information for a specific SyncThing instance, including connectivity, version, uptime, and resource usage.
    - `get_system_stats`: Get SyncThing system statistics, including version, uptime, memory usage, and the unique device ID.
    - `get_sync_status`: Get detailed synchronization status, state, and completion percentage for a specific folder or device.
    - `manage_folders`: List all configured SyncThing folders, showing their IDs, labels, paths, and paused status.
    - `browse_folder`: Browse the contents of a synced folder, listing files and subdirectories with optional prefix and recursion depth control.
    - `configure_sharing`: Share or unshare a specific folder with a remote device.
    - `manage_ignores`: Manage SyncThing ignore patterns (.stignore). Supports getting current patterns, setting a new list, or appending to the existing list.
    - `manage_devices`: Manage SyncThing devices, including listing, adding, removing, pausing, resuming, and approving pending devices.
    - `maintain_system`: Perform system maintenance: force a rescan of folders, restart the SyncThing service, or clear internal errors.
    - `replicate_config`: Replicate folder and device configurations from one SyncThing instance to another for easy synchronization setup.

## :package: Installation

### Homebrew

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

### :remote: HTTP/SSE Remote Access

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
| `--host` | `SYNCTHING_HOST` | SyncThing instance host | `localhost` |
| `--port` | `SYNCTHING_PORT` | SyncThing instance port | `8384` |
| `--api-key` | `SYNCTHING_API_KEY` | SyncThing API key (supports `keyring:...` and `encrypted:...`) | - |
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
