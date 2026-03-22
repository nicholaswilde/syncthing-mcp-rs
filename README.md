# 🔄 SyncThing MCP Server (Rust) 🤖

[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)

> [!WARNING]
> This project is currently in active development (v0.1.0) and is **not production-ready**. Features may change, and breaking changes may occur without notice. **Use this MCP server at your own risk.**

A Rust implementation of a SyncThing [MCP (Model Context Protocol) server](https://modelcontextprotocol.io/docs/getting-started/intro). This server connects to one or more SyncThing instances and exposes tools to monitor and manage file synchronization via the Model Context Protocol.

## ✨ Features

- **Multi-Transport Support:**
  - **Stdio:** Default transport for local integrations (e.g., Claude Desktop).
- **Multi-Instance Management:** Manage and target multiple SyncThing instances from a single MCP server. Tools accept an optional `instance` argument (name or index).
- **Robust Configuration:** Supports configuration via CLI arguments, environment variables, and configuration files (TOML, YAML, JSON).
- **Authentication:** Connects to SyncThing using API Key (`X-API-Key`).
- **Token Optimization:** Consolidated tools into functional groups to optimize AI context window usage.
  - **Tools:**
    - `get_system_stats`: Retrieve SyncThing version, uptime, and resource usage.
    - `manage_folders`: List and monitor shared folders.

## 🔨 Build

To build the project, you need a Rust toolchain installed.

### Local Build

```bash
# Build in release mode
task build:local
```

The binary will be available at `target/release/syncthing-mcp-rs`.

## 🚀 Usage

### ⌨️ Command Line Interface

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

### 📁 Configuration File

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

### 🤖 Configuration Example (Claude Desktop)

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

## 🧪 Testing

The project uses [go-task](https://taskfile.dev/) for development tasks.

```bash
# Run all checks (format, lint, unit tests)
task test

# Run unit tests only
cargo test
```

## ⚖️ License

[Apache License 2.0](LICENSE)

## ✍️ Author

This project was started in 2026 by [Nicholas Wilde][2].

[2]: <https://github.com/nicholaswilde/>
