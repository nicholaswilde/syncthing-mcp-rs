# Implementation Plan: Device Management (manage_devices_20260321)

## Phase 1: API Client Implementation [checkpoint: 0f819d3]
- [x] Task: Add device-related endpoints to `api/client.rs`. (0f819d3)
- [x] Task: Define device models in `api/models.rs`. (0f819d3)
- [x] Task: Unit tests for API client methods. (0f819d3)

## Phase 2: MCP Tool Implementation
- [ ] Task: Implement `manage_devices` tool in `src/tools/devices.rs`.
- [ ] Task: Register tool in `src/mcp/server.rs`.
- [ ] Task: Implement summarized output for device listing.

## Phase 3: Docker Integration Tests
- [ ] Task: Add integration tests in `tests/docker_integration_test.rs`.
- [ ] Task: Verify add/remove/pause/resume cycle against a real instance.
