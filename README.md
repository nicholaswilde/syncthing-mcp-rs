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
- **Multi-Instance Management:** Manage and target multiple SyncThing instances from a single MCP server. Tools accept an optional `instance` argument (name or index).
- **Robust Configuration:** Supports configuration via CLI arguments, environment variables, and configuration files (TOML, YAML, JSON).
- **Authentication:** Connects to SyncThing using API Key (`X-API-Key`).
- **Token Optimization:** Consolidated tools into functional groups to optimize AI context window usage.
  - **Tools:**
    - `get_system_stats`: Retrieve SyncThing version, uptime, and resource usage.
    - `get_sync_status`: Get detailed synchronization status and completion percentage.
    - `manage_folders`: List and monitor shared folders.
    - `configure_sharing`: Share or unshare a folder with a device.
    - `manage_ignores`: Manage SyncThing ignore patterns (`.stignore`).
    - `manage_devices`: Manage SyncThing devices (list, add, remove, pause, resume).
    - `maintain_system`: Perform maintenance tasks (rescan, restart, clear errors).

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

The binary will be available at `target/release/syncthing-mcp-rs`.

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
./target/release/syncthing-mcp-rs --host "localhost" --port 8384 --api-key "your-api-key"
```

#### Available Arguments

| Argument | Environment Variable | Description | Default |
| :--- | :--- | :--- | :--- |
| `-c, --config` | - | Path to configuration file | `config.toml` |
| `--host` | `SYNCTHING_HOST` | SyncThing instance host | `localhost` |
| `--port` | `SYNCTHING_PORT` | SyncThing instance port | `8384` |
| `--api-key` | `SYNCTHING_API_KEY` | SyncThing API key | - |
| `--transport` | `SYNCTHING_MCP_TRANSPORT` | Transport mode (`stdio`) | `stdio` |
| `--no-verify-ssl` | `SYNCTHING_NO_VERIFY_SSL` | Disable SSL certificate verification | `true` |
| `--log-level` | `SYNCTHING_LOG_LEVEL` | Log level (`info`, `debug`, etc.) | `info` |
| - | `SYNCTHING_INSTANCES__<N>__<FIELD>` | Configuration for multiple instances | - |

### :file_folder: Configuration File

The server automatically looks for `config.toml`, `config.yaml`, or `config.json` in the current directory and `~/.config/syncthing-mcp-rs/`.

#### Multi-Instance Configuration

```toml
# Default instance
host = "localhost"
port = 8384
api_key = "your-api-key"

# OR use the instances list
[[instances]]
name = "primary"
url = "http://localhost:8384"
api_key = "your-api-key"

[[instances]]
name = "remote"
url = "https://sync.example.com"
api_key = "remote-api-key"
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
