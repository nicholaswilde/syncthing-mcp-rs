# Implementation Plan: Feature Gaps

## Phase 1: Event System
- [x] Implement an event listener task in `src/mcp/server.rs`. `db9cb21`
- [x] Map SyncThing events to MCP notification methods. `db9cb21`
- [x] Add event polling to the main loop. `db9cb21`

## Phase 2: File Browser
- [ ] Implement `browse_folder` tool in a new `src/tools/browser.rs`.
- [ ] Add the tool to the registry.

## Phase 3: Device Discovery
- [ ] Extend `manage_devices` with "discover" and "approve" actions.
- [ ] Update tool metadata and documentation.

## Phase 4: Validation
- [ ] Test the file browser with a real folder.
- [ ] Mock the Event API to verify notifications.
