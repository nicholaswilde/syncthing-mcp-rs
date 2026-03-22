# Implementation Plan: Feature Gaps

## Phase 1: Event System
- [ ] Implement an event listener task in `src/mcp/server.rs`.
- [ ] Map SyncThing events to MCP notification methods.
- [ ] Add event polling to the main loop.

## Phase 2: File Browser
- [ ] Implement `browse_folder` tool in a new `src/tools/browser.rs`.
- [ ] Add the tool to the registry.

## Phase 3: Device Discovery
- [ ] Extend `manage_devices` with "discover" and "approve" actions.
- [ ] Update tool metadata and documentation.

## Phase 4: Validation
- [ ] Test the file browser with a real folder.
- [ ] Mock the Event API to verify notifications.
