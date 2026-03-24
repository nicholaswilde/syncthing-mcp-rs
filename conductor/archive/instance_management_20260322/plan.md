# Implementation Plan: Instance Management Tools (instance_management_20260322)

## Phase 1: Core Monitoring Logic [checkpoint: 9071e60]
- [x] Task: Implement a health check function that tests connectivity to the SyncThing REST API. 79a6f95
- [x] Task: Create a health check result model (status, latency, errors, version). 79a6f95

## Phase 2: Tool Implementation
- [x] Task: Develop the `list_instances` tool with status badges (Online, Offline, Error). 81a910a
- [x] Task: Implement the `get_instance_health` tool for detailed diagnostics. 81a910a
- [x] Task: Support for checking all instances in a single tool call. 81a910a

## Phase 3: Enhanced Diagnostics
- [x] Task: Integrate resource monitoring (CPU/RAM) into instance health checks. 81a910a
- [ ] Task: Implement basic connectivity troubleshooting (e.g., DNS check, port accessibility).
- [x] Task: Improve formatting of health reports for the MCP client. 81a910a

## Phase 4: Validation
- [x] Task: Unit tests for connectivity check logic with mock server responses. 81a910a
- [x] Task: Integration tests for multi-instance configuration parsing. 81a910a
- [x] Task: Verify tool behavior with real-world instance failures (simulated via Docker). 81a910a
