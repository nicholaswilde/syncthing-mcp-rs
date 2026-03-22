# Track Specification: Docker Integration Tests (docker_integration_20260321)

## Overview
Implement automated integration tests that launch a real SyncThing instance using Docker (via `testcontainers-rs`) to verify the end-to-end functionality of the MCP server's tools and API client.

## Functional Requirements
- **FR-1: Docker Lifecycle Management**: Use the `testcontainers` crate to programmatically start and stop a SyncThing container for tests.
- **FR-2: Environment Configuration**: Configure the container with a known API key and ensure the web UI/API port (8384) is accessible.
- **FR-3: Integration Test Suite**: Implement a test file `tests/docker_integration_test.rs` that verifies:
    - Successful connection and authentication.
    - `get_system_stats` returns valid data from a real instance.
    - `manage_folders (list)` correctly retrieves folder information.
    - Handling of invalid API keys or unreachable instances.

## Non-Functional Requirements
- **NFR-1: Reliability**: Tests should be idempotent and not leave dangling containers.
- **NFR-2: Speed**: Optimize container startup time (e.g., by using a specific tag or minimal configuration).
- **NFR-3: Observability**: Ensure container logs are accessible or integrated into the test output for easier debugging.

## Technical Details
- **Base Image**: `syncthing/syncthing:latest`
- **Tooling**: `testcontainers-rs`, `tokio`, `reqwest`.
- **Reference**: `https://github.com/nicholaswilde/adguardhome-mcp-rs/blob/main/tests/docker_integration_test.rs`

## Acceptance Criteria
- [ ] New test file `tests/docker_integration_test.rs` exists.
- [ ] `cargo test --test docker_integration_test` passes when Docker is available.
- [ ] Container is automatically pulled and started during the test run.
- [ ] Container is automatically stopped and removed after tests complete.
