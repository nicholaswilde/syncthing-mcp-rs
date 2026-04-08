# Tech Stack - SyncThing MCP Server (Rust)

## Core Language & Protocol
- **Rust**: Primary language (99.8%) for performance and safety.
- **Model Context Protocol (MCP)**: For interacting with LLMs.
- **Transports**: Supports both `stdio` (local) and `HTTP/SSE` (remote).

## Build & Development Tools
- Task Runner: `go-task` (via `Taskfile.yml`) for build, test, and deployment automation.
- Cross-Compilation: `cross` for building `amd64`, `arm64`.
- Version Control: `git` for configuration backup and management.
- Package Manager: `cargo` (Rust standard).
- MCP Testing: `mcp-inspector` for verifying MCP tool definitions and responses.

## Configuration Management
- Formats: Optimized support for **TOML** (YAML/JSON disabled for size).
- Hierarchy: Configuration via CLI arguments, environment variables, and config files (e.g., `config.toml`).
- Backups: Git-Sync integration for automated configuration versioning and rollbacks.
- Multi-Instance Support: Built-in logic for managing multiple SyncThing instances.


## Security & Secrets
- **Secrets Encryption**: OS Keyring integration, external secret store support (HashiCorp Vault, AWS Secrets Manager), and authenticated encryption (ChaCha20-Poly1305).
- **Environment Security**: Secure credential resolution from OS-level and external secret stores.
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
- **tokio-retry**: Exponential backoff and retry strategy for async tasks.
- **chrono**: Date and time handling for event filtering and timelines.
- **reqwest**: For communicating with the SyncThing REST API.
- **serde**, **serde_json**, **toml**: For configuration and API parsing.
- **keyring**: OS Keyring integration.
- **chacha20poly1305**: Authenticated encryption for sensitive fields.
- **native-tls**: Minimal TLS implementation for reduced binary size.
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
- **vaultrs**: Modern, asynchronous client for HashiCorp Vault.
- **aws-sdk-secretsmanager**: AWS SDK for interacting with Secrets Manager.
- **async-trait**: Support for asynchronous functions in traits.
- **uuid**: Unique identifier generation.
- **url**: URL parsing and manipulation.
- **trash**: Cross-platform file deletion to trash.
- **async-recursion**: Procedural macro for recursive async functions.
- **similar**: Textual diffing engine.
- **serde_json_diff**: Semantic JSON diffing.
- **serde_yaml_ng**: YAML support for semantic diffing.
