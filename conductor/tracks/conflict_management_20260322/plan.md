# Implementation Plan: Sync Conflict Management (conflict_management_20260322)

## Phase 1: Conflict Detection logic
- [x] Task: Implement a conflict file scanner that recognizes SyncThing's naming pattern. (887d447)
- [x] Task: Create a model for conflict file information (original path, conflict path, device ID, timestamp). (887d447)

## Phase 2: Tool Development
- [ ] Task: Implement the `list_conflicts` tool.
- [ ] Task: Develop the `resolve_conflict` tool with basic "keep original" or "keep conflict" actions.
- [ ] Task: Implement a `delete_conflict` tool for simple cleanup of unwanted conflict files.

## Phase 3: Safety & UI Enhancements
- [ ] Task: Add "dry-run" mode to resolution tools to preview changes.
- [ ] Task: Implement backup logic (e.g., move to trash) before overwriting files.
- [ ] Task: Provide more detailed file comparisons in `list_conflicts` output.

## Phase 4: Validation
- [ ] Task: Unit tests for conflict filename regex and parsing.
- [ ] Task: Integration tests with temporary test files and directories.
- [ ] Task: Verify tool behavior when files are locked or inaccessible.
