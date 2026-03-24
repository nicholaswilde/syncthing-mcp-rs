# Track Specification: Event Notifications (event_notifications_20260322)

## Overview
Implement a background polling loop that monitors SyncThing's `/rest/events` endpoint for all configured instances. These events will be filtered for relevance and pushed to connected MCP clients as notifications.

## Scope
- Develop a background task manager for event polling.
- Implement efficient long-polling of the SyncThing events API.
- Create a filtering mechanism to select relevant event types (e.g., folder completion, device connection, sync errors).
- Integrate event push notifications into the MCP server (supporting both stdio and future SSE transports).
- Support per-instance event tracking and deduplication.

## Success Criteria
- [ ] Background polling tasks successfully start for all configured SyncThing instances.
- [ ] New SyncThing events are correctly captured and processed.
- [ ] Relevant events are pushed to the MCP client as notifications.
- [ ] Polling handles instance downtime and reconnection gracefully.
- [ ] Performance testing ensures low overhead even with multiple instances.
