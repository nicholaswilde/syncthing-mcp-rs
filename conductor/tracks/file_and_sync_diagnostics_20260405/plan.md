# Implementation Plan: File & Sync Diagnostics

## Phase 1: API Client Implementation
- [x] Task 1.1: Define response models for `/rest/db/file` and `/rest/db/needs` in `src/api/models.rs` f3f7476
- [x] Task 1.2: Implement `get_file_info` method in `src/api/client.rs` 12276a4
- [x] Task 1.3: Implement `get_folder_needs` method in `src/api/client.rs` 3c1e27f
- [x] Task 1.4: Add live API verification script and task e55feab

## Phase 2: MCP Tools Implementation [checkpoint: 933f6ec]
- [x] Task 2.1: Create new tool `get_file_info` in `src/tools/` 933f6ec
- [x] Task 2.2: Create new tool `get_folder_needs` in `src/tools/` 933f6ec
- [x] Task 2.3: Register tools in `src/tools/mod.rs` 933f6ec

## Phase 3: Testing & Documentation
- [ ] Task 3.1: Write unit tests for new API methods
- [ ] Task 3.2: Write Docker integration tests for new MCP tools
- [ ] Task 3.3: Update `README.md` with new tool documentation
