# Implementation Plan: Build core SyncThing MCP server functionality (core_functionality_20260321)

## Phase 1: Project Scaffolding [checkpoint: bc85586]
- [x] Task: Initialize Rust project and directory structure. (6880b10)
    - [x] Run `cargo init` and create basic folder structure (`src/api`, `src/mcp`, `src/config`).
    - [x] Create `Taskfile.yml` with basic build and test tasks.
    - [x] Configure `Cargo.toml` with necessary dependencies.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Project Scaffolding' (Protocol in workflow.md) (bc85586)

## Phase 2: Configuration & API Client
- [x] Task: Implement configuration loading and validation. (c5154d3)
    - [x] Create `src/config/mod.rs` to handle TOML/YAML/JSON parsing.
    - [x] Define `InstanceConfig` and `AppConfig` structs with validation.
- [ ] Task: Develop asynchronous SyncThing REST API client.
    - [ ] Create `src/api/client.rs` using `reqwest`.
    - [ ] Implement `get_system_info` and `list_folders` API calls.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Configuration & API Client' (Protocol in workflow.md)

## Phase 3: Core MCP Implementation
- [ ] Task: Implement core MCP server logic.
    - [ ] Create `src/mcp/server.rs` with `stdio` transport.
    - [ ] Register `get_system_stats` tool in the MCP server.
    - [ ] Register `manage_folders` tool for listing folder status.
- [ ] Task: Add basic logging and error handling.
    - [ ] Integrate `tracing` and `tracing-subscriber`.
    - [ ] Implement custom error types and `From` conversions.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Core MCP Implementation' (Protocol in workflow.md)

## Phase 4: Quality & Validation
- [ ] Task: Implement comprehensive unit tests.
    - [ ] Create mock API tests for `src/api/client.rs`.
    - [ ] Add integration tests for MCP tool responses.
- [ ] Task: Verify tool definitions and documentation.
    - [ ] Use `mcp-inspector` to validate tool descriptions and schemas.
    - [ ] Update `README.md` with setup instructions and usage examples.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Quality & Validation' (Protocol in workflow.md)
