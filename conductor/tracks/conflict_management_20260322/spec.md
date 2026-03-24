# Track Specification: Sync Conflict Management (conflict_management_20260322)

## Overview
Provide specialized MCP tools to help users and AI agents identify and resolve SyncThing conflict files (e.g., `filename.sync-conflict-20230101-120000-DEVICE.ext`).

## Scope
- Implement a `list_conflicts` tool to scan folders for conflict files.
- Develop a `resolve_conflict` tool with options to:
    - Keep the conflict version (overwrite the original).
    - Keep the original version (delete the conflict).
    - Delete both (if appropriate).
- Add support for recursive scanning of conflict files within shared folders.
- Provide file metadata (size, modification time) to help with resolution decisions.

## Success Criteria
- [ ] `list_conflicts` accurately identifies conflict files in various folder structures.
- [ ] `resolve_conflict` successfully performs file operations on conflict files.
- [ ] Resolution actions are logged and traceable.
- [ ] Error handling prevents data loss during resolution.
- [ ] Automated tests verify conflict detection logic.
