# Implementation Plan: Device Management (manage_devices_20260321)

## Phase 1: API Client Implementation [checkpoint: 0f819d3]
- [x] Task: Add device-related endpoints to `api/client.rs`. (0f819d3)
- [x] Task: Define device models in `api/models.rs`. (0f819d3)
- [x] Task: Unit tests for API client methods. (0f819d3)

## Phase 2: MCP Tool Implementation [checkpoint: 4eb6957]
- [x] Task: Implement `manage_devices` tool in `src/tools/devices.rs`. (4eb6957)
- [x] Task: Register tool in `src/mcp/server.rs`. (4eb6957)
- [x] Task: Implement summarized output for device listing. (4eb6957)

## Phase 3: Docker Integration Tests [checkpoint: 712b9d8]
- [x] Task: Add integration tests in `tests/docker_integration_test.rs`. (712b9d8)
- [x] Task: Verify add/remove/pause/resume cycle against a real instance. (712b9d8)
