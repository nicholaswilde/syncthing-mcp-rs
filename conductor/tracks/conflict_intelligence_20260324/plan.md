# Implementation Plan: Advanced Conflict Intelligence (conflict_intelligence_20260324)

## Phase 1: Core Diffing Engine
- [ ] Task: Research and select appropriate diffing libraries for Rust.
- [ ] Task: Implement basic textual diff extraction for text files.
- [ ] Task: Implement semantic diffing for JSON/YAML files.

## Phase 2: Preview Generation
- [ ] Task: Create a preview generator that shows the result of a conflict resolution.
- [ ] Task: Support "keep_original" and "keep_conflict" preview modes.

## Phase 3: MCP Tool Integration
- [ ] Task: Create new MCP tools for conflict diffing and previewing.
- [ ] Task: Update `resolve_conflict` to support a preview step.

## Phase 4: Validation
- [ ] Task: Unit tests for diffing logic with various file formats.
- [ ] Task: Integration tests for the new MCP tools.
- [ ] Task: End-to-end testing with real SyncThing conflict scenarios.
