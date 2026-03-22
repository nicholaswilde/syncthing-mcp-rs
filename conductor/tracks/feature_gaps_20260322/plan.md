# Implementation Plan: Feature Gaps

## Phase 1: Event System [checkpoint: f38b761]
- [x] Implement an event listener task in `src/mcp/server.rs`. `db9cb21`
- [x] Map SyncThing events to MCP notification methods. `db9cb21`
- [x] Add event polling to the main loop. `db9cb21`

## Phase 2: File Browser [checkpoint: 0a4460c]
- [x] Implement `browse_folder` tool in a new `src/tools/browser.rs`. `8530636`
- [x] Add the tool to the registry. `8530636`

## Phase 3: Device Discovery
- [x] Extend `manage_devices` with "discover" and "approve" actions. `3341c1d`
- [x] Update tool metadata and documentation. `3341c1d`

## Phase 4: Validation
- [ ] Test the file browser with a real folder.
- [ ] Mock the Event API to verify notifications.
