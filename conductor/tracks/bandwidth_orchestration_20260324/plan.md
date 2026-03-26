# Implementation Plan: Bandwidth Orchestration (bandwidth_orchestration_20260324)

## Phase 1: Bandwidth Controller
- [x] Task: Implement a controller to update upload/download limits across instances. 07da1d6
- [x] Task: Support global limits for all instances or per-instance limits. 07da1d6

## Phase 2: Performance Profiles
- [x] Task: Define a format for performance profiles. 604e630
- [x] Task: Implement a profile manager that applies limits based on chosen profiles. a1b817a
- [ ] Task: Support scheduled profile activation.

## Phase 3: MCP Tool Integration
- [ ] Task: Create new MCP tools `set_bandwidth_limits` and `set_performance_profile`.
- [ ] Task: Implement status reporting for current bandwidth limits and active profiles.

## Phase 4: Validation
- [ ] Task: Unit tests for bandwidth limit calculations and profile logic.
- [ ] Task: Integration tests with SyncThing instances.
- [ ] Task: End-to-end testing of profile scheduling and limit updates.
