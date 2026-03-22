# Track Specification: Ignore Pattern Management (manage_ignores_20260321)

## Overview
Implement the `manage_ignores` tool to manage Syncthing ignore patterns (`.stignore`).

## Functional Requirements
- **FR-1: Get Ignores**: Retrieve current ignore patterns for a folder.
- **FR-2: Append Ignores**: Add new patterns to the current list.
- **FR-3: Set Ignores**: Overwrite all patterns for a folder.

## Non-Functional Requirements
- **NFR-1: Token Optimization**: Summarize long ignore lists in the output.
- **NFR-2: Validation**: Validate patterns before applying them.

## Acceptance Criteria
- [ ] `manage_ignores(action="get")` returns the current patterns.
- [ ] `manage_ignores(action="append")` adds patterns without losing existing ones.
- [ ] `manage_ignores(action="set")` overwrites the ignore list correctly.
- [ ] Docker integration tests verify ignore management against a live Syncthing instance.
