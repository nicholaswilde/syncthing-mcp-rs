# Implementation Plan: Network Performance Analytics

## Phase 1: Data Model Enhancements [checkpoint: 5d7764d]
- [x] Refine `SystemConnections` and related models in `src/api/models.rs` to capture more granular data (types, address, etc.). [a50c50f]
- [x] Ensure backward compatibility with existing summary tools. [a50c50f]

## Phase 2: Tool Implementation
- [x] Update `get_system_connections` tool to provide a more detailed "Analytics" mode. [61000d3]
- [x] Create a new diagnostic tool `diagnose_network_issues` that analyzes discovery vs. connection states. [06364dd]
- [x] Register tools in `src/tools/mod.rs`. [06364dd]

## Phase 3: Verification
- [ ] Write unit and integration tests.
- [ ] Update documentation.
