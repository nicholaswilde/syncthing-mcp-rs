# Implementation Plan: Advanced Sync Operations

## Phase 1: API Client Implementation
- [x] Define response models for `/rest/db/completion` in `src/api/models.rs`. [61d10c0]
- [x] Implement `set_file_priority()` method in `src/api/client.rs`. [e5a6333]
- [x] Implement `get_device_completion()` method in `src/api/client.rs`. [5a44d1f]

## Phase 2: MCP Tools Implementation
- [x] Create new tool `set_file_priority`. [119c098]
- [x] Create new tool `get_device_sync_status` (per-device). [0ac5900]
- [x] Update `inspect_folder` to optionally include per-device completion if requested. [0ac215c]
- [~] Register tools in `src/tools/mod.rs`.

## Phase 3: Verification
- [ ] Write unit and integration tests.
- [ ] Update documentation.
