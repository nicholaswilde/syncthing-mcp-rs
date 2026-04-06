# Implementation Plan: Configuration & Error Management

## Phase 1: API Client Implementation
- [x] Define response models for `/rest/system/config/insync` and `/rest/system/error` in `src/api/models.rs`.
- [x] Implement `is_config_insync()` and `get_errors()` methods in `src/api/client.rs`. [296c3f4]

## Phase 2: MCP Tools Implementation
- [ ] Create new tool `get_system_errors` and integrate config sync checks in `src/tools/system.rs`.
- [ ] Register tools in `src/tools/mod.rs`.

## Phase 3: Verification
- [ ] Write unit and integration tests.
- [ ] Update documentation.
