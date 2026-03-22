# Tech Stack - SyncThing MCP Server (Rust)

## Core Language & Protocol
- **Rust**: Primary language (99.8%) for performance and safety.
- **Model Context Protocol (MCP)**: For interacting with LLMs.
- **Transports**: Supports both `stdio` (local) and `HTTP/SSE` (remote).

## Build & Development Tools
- **Task Runner**: `go-task` (via `Taskfile.yml`) for build, test, and deployment automation.
- **Cross-Compilation**: `cross` for building `amd64`, `arm64`, and `armv7` architectures.
- **Package Manager**: `cargo` (Rust standard).
- **MCP Testing**: `mcp-inspector` for verifying MCP tool definitions and responses.

## Configuration Management
- **Formats**: Multi-format support for **TOML**, **YAML**, and **JSON**.
- **Hierarchy**: Configuration via CLI arguments, environment variables, and config files (e.g., `config.toml`).
- **Multi-Instance Support**: Built-in logic for managing multiple SyncThing instances.

## Security & Secrets
- **Secrets Encryption**: `sops` (Mozilla SOPS) for managing encrypted secrets.
- **Environment Security**: Support for encrypted environment files (`.env.enc`).
- **Authentication**: Bearer Token for HTTP transport security; API Key support for SyncThing backend communication.

## Testing & Quality Assurance
- **Code Coverage**: `cargo-llvm-cov` for detailed analysis.
- **Integration Testing**: Automated end-to-end testing using `testcontainers-rs` and real SyncThing Docker instances.
- **CI/CD**: Integration with GitHub Actions and Coveralls.io.

## Containerization
- **Docker**: Standard `Dockerfile` for containerized deployment.
- **Orchestration**: `compose.yaml` for local development and integration testing.

## Principal Rust Dependencies (Inferred)
- **tokio**: Asynchronous runtime.
- **reqwest**: For communicating with the SyncThing REST API.
- **serde**, **serde_json**, **toml**, **serde_yaml**: For multi-format configuration parsing.
- **clap**: For robust CLI argument parsing.
- **tracing** / **log**: For configurable logging levels.
- **anyhow**: Flexible error handling.
- **thiserror**: Custom error types with derive macros.
- **axum**: Web framework for HTTP/SSE transport (if implemented).
- **config**: Hierarchical configuration management.
- **dashmap**: Concurrent associative array.
- **futures**: Utilities for asynchronous programming.
- **testcontainers**: Programmatic Docker lifecycle management for tests.
- **testcontainers-modules**: Ready-to-use Docker images for testcontainers.
- **uuid**: Unique identifier generation.
- **url**: URL parsing and manipulation.
