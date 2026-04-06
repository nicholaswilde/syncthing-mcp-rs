# Implementation Plan: System Maintenance & Lifecycle

## Phase 1: API Client Implementation
- [x] Task 1.1: Define response models for `/rest/system/upgrade` and `/rest/system/ping` in `src/api/models.rs` b7e1cb0
- [x] Task 1.2: Implement `check_upgrade` method in `src/api/client.rs` b70cd2c
- [x] Task 1.3: Implement `perform_upgrade` method in `src/api/client.rs` d34e9f7
- [x] Task 1.4: Implement `ping` method in `src/api/client.rs` bfd2688

## Phase 2: MCP Tools Implementation [checkpoint: 8b3575b]
- [x] Task 2.1: Create new tool `check_upgrade` in `src/tools/` 8b3575b
- [x] Task 2.2: Create new tool `perform_upgrade` in `src/tools/` 8b3575b
- [x] Task 2.3: Create new tool `ping_instance` in `src/tools/` 8b3575b
- [x] Task 2.4: Register tools in `src/tools/mod.rs` 8b3575b

## Phase 3: Testing & Documentation [checkpoint: 9fe6ca0]
- [x] Task 3.1: Write unit tests for new API methods 9fe6ca0
- [x] Task 3.2: Write Docker integration tests for new MCP tools 9fe6ca0
- [x] Task 3.3: Update `README.md` with new tool documentation 9fe6ca0
