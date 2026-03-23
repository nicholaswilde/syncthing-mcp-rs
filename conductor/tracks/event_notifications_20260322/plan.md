# Implementation Plan: Event Notifications (event_notifications_20260322)

## Phase 1: Event Client & Polling [checkpoint: 558a69c]
- [x] Task: Implement the `/rest/events` API call in `src/api/client.rs`. 5294549
- [x] Task: Create a dedicated `EventManager` background task. f5df6ba
- [x] Task: Implement a long-polling loop with proper `since` parameter management. f5df6ba

## Phase 2: Processing & Filtering
- [x] Task: Define event models for various SyncThing event types. 6cd4e6a
- [x] Task: Implement a configurable filter to select which events to forward as notifications. cb9e60f
- [ ] Task: Add basic event transformation to make them more "human-readable" for the MCP client.

## Phase 3: MCP Server Integration
- [ ] Task: Extend the MCP server to support pushing notifications to clients.
- [ ] Task: Connect the `EventManager` to the MCP server's notification channel.
- [ ] Task: Ensure notification delivery works correctly for multiple instances.

## Phase 4: Validation
- [ ] Task: Create unit tests for event parsing and filtering.
- [ ] Task: Implement integration tests with mock event sequences.
- [ ] Task: Verify real-time notification delivery using the `mcp-inspector`.
