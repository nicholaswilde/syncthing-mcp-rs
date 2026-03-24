# Implementation Plan: Instance Management Tools (instance_management_20260322)

## Phase 1: Core Monitoring Logic
- [x] Task: Implement a health check function that tests connectivity to the SyncThing REST API. 79a6f95
- [x] Task: Create a health check result model (status, latency, errors, version). 79a6f95

## Phase 2: Tool Implementation
- [ ] Task: Develop the `list_instances` tool with status badges (Online, Offline, Error).
- [ ] Task: Implement the `get_instance_health` tool for detailed diagnostics.
- [ ] Task: Support for checking all instances in a single tool call.

## Phase 3: Enhanced Diagnostics
- [ ] Task: Integrate resource monitoring (CPU/RAM) into instance health checks.
- [ ] Task: Implement basic connectivity troubleshooting (e.g., DNS check, port accessibility).
- [ ] Task: Improve formatting of health reports for the MCP client.

## Phase 4: Validation
- [ ] Task: Unit tests for connectivity check logic with mock server responses.
- [ ] Task: Integration tests for multi-instance configuration parsing.
- [ ] Task: Verify tool behavior with real-world instance failures (simulated via Docker).
