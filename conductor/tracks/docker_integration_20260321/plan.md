# Implementation Plan: Docker Integration Tests (docker_integration_20260321)

## Phase 1: Test Environment Setup [checkpoint: 5e9b167]
- [x] Task: Configure `Cargo.toml` for integration testing. (9f81355)
    - [x] Add `testcontainers` and `testcontainers-modules` (if needed) to `[dev-dependencies]`.
- [x] Task: Create helper for SyncThing container lifecycle. (837150a)
    - [x] Implement a `SyncThingContainer` wrapper using `testcontainers`.
    - [x] Define methods to start the container with a preset API key.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Test Environment Setup' (Protocol in workflow.md) (5e9b167)

## Phase 2: Core Integration Tests [checkpoint: 904ebeb]
- [x] Task: Implement `get_system_stats` integration test. (6501f93)
    - [x] Write failing test in `tests/docker_integration_test.rs` that calls `get_system_stats` against the container.
    - [x] Implement enough of the test to verify connectivity and basic parsing.
- [x] Task: Implement `manage_folders` integration test. (71f8e76)
    - [x] Write failing test that verifies folder listing from the real SyncThing instance.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Core Integration Tests' (Protocol in workflow.md) (904ebeb)

## Phase 3: Error Handling & Cleanup
- [x] Task: Test authentication failure. (c565812)
    - [x] Write a test case using an invalid API key against the running container.
- [ ] Task: Ensure proper cleanup.
    - [ ] Verify that containers are stopped even if tests fail (using `testcontainers` RAII).
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Error Handling & Cleanup' (Protocol in workflow.md)
