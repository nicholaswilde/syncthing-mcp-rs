# Implementation Plan: Network & Discovery Troubleshooting

## Phase 1: API Client Implementation
- [x] Task 1.1: Define response models for `/rest/system/discovery` in `src/api/models.rs` 922d497
- [x] Task 1.2: Implement `get_discovery_status` method in `src/api/client.rs` b967eca

## Phase 2: MCP Tools Implementation [checkpoint: 6d99d26]
- [x] Task 2.1: Create new tool `get_discovery_status` in `src/tools/` 6d99d26
- [x] Task 2.2: Register tool in `src/tools/mod.rs` 6d99d26

## Phase 3: Testing & Documentation
- [ ] Task 3.1: Write unit tests for new API method
- [ ] Task 3.2: Write Docker integration tests for new MCP tool
- [ ] Task 3.3: Update `README.md` with new tool documentation
