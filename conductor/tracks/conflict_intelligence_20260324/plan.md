# Implementation Plan: Advanced Conflict Intelligence (conflict_intelligence_20260324)

## Phase 1: Core Diffing Engine [checkpoint: a82e877]
- [x] Task: Research and select appropriate diffing libraries for Rust. (0f1fab5)
- [x] Task: Implement basic textual diff extraction for text files. (d3e964d)
- [x] Task: Implement semantic diffing for JSON/YAML files. (709e516)

## Phase 2: Preview Generation [checkpoint: c90d4c1]
- [x] Task: Create a preview generator that shows the result of a conflict resolution. (4a2e81c)
- [x] Task: Support "keep_original" and "keep_conflict" preview modes. (4a2e81c)

## Phase 3: MCP Tool Integration
- [ ] Task: Create new MCP tools for conflict diffing and previewing.
- [ ] Task: Update `resolve_conflict` to support a preview step.

## Phase 4: Validation
- [ ] Task: Unit tests for diffing logic with various file formats.
- [ ] Task: Integration tests for the new MCP tools.
- [ ] Task: End-to-end testing with real SyncThing conflict scenarios.
