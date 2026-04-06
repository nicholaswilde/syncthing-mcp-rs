# Implementation Plan: Advanced Sync Operations

## Phase 1: API Client Implementation
- [x] Define response models for `/rest/db/completion` in `src/api/models.rs`. [61d10c0]
- [ ] Implement `set_file_priority()` method in `src/api/client.rs`.
- [ ] Implement `get_device_completion()` method in `src/api/client.rs`.

## Phase 2: MCP Tools Implementation
- [ ] Create new tool `set_file_priority`.
- [ ] Create new tool `get_device_sync_status` (per-device).
- [ ] Update `inspect_folder` to optionally include per-device completion if requested.
- [ ] Register tools in `src/tools/mod.rs`.

## Phase 3: Verification
- [ ] Write unit and integration tests.
- [ ] Update documentation.
