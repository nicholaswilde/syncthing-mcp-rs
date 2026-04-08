# Implementation Plan: Event Timeline Analysis

## Phase 1: API Client & Logic
- [x] Implement time-based event filtering logic in `SyncThingClient`. (7a0ea64)
- [ ] Define summary models for different event types to reduce token usage in timelines.

## Phase 2: MCP Tools Implementation
- [ ] Create new tool `get_event_timeline`.
- [ ] Implement basic "Intelligence" to highlight critical event patterns.
- [ ] Register tools in `src/tools/mod.rs`.

## Phase 3: Verification
- [ ] Write unit and integration tests.
- [ ] Update documentation.
