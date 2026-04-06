# Implementation Plan: Configuration & Error Management

## Phase 1: API Client Implementation
- [x] Define response models for `/rest/system/config/insync` and `/rest/system/error` in `src/api/models.rs`.
- [x] Implement `is_config_insync()` and `get_errors()` methods in `src/api/client.rs`. [296c3f4]

## Phase 2: MCP Tools Implementation
- [x] Create new tool `get_system_errors` and integrate config sync checks in `src/tools/system.rs`. [586ea2d]
- [x] Register tools in `src/tools/mod.rs`. [586ea2d]

## Phase 3: Verification
- [x] Write unit and integration tests. [8173b3d]
- [x] Update documentation. [8173b3d]
