# Implementation Plan: Event Notifications (event_notifications_20260322)

## Phase 1: Event Client & Polling [checkpoint: 558a69c]
- [x] Task: Implement the `/rest/events` API call in `src/api/client.rs`. 5294549
- [x] Task: Create a dedicated `EventManager` background task. f5df6ba
- [x] Task: Implement a long-polling loop with proper `since` parameter management. f5df6ba

## Phase 2: Processing & Filtering [checkpoint: 0af18d3]
- [x] Task: Define event models for various SyncThing event types. 6cd4e6a
- [x] Task: Implement a configurable filter to select which events to forward as notifications. cb9e60f
- [x] Task: Add basic event transformation to make them more "human-readable" for the MCP client. 8d623b8

## Phase 3: MCP Server Integration
- [x] Task: Extend the MCP server to support pushing notifications to clients. 41eda66
- [x] Task: Connect the `EventManager` to the MCP server's notification channel. 3b3ff64
- [x] Task: Ensure notification delivery works correctly for multiple instances. 6b6029b

## Phase 4: Validation
- [ ] Task: Create unit tests for event parsing and filtering.
- [ ] Task: Implement integration tests with mock event sequences.
- [ ] Task: Verify real-time notification delivery using the `mcp-inspector`.
