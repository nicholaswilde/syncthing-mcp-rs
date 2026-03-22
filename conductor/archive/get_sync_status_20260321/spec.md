# Track Specification: Detailed Sync Status (get_sync_status_20260321)

## Overview
Implement the `get_sync_status` tool to monitor the synchronization progress of folders and devices.

## Functional Requirements
- **FR-1: Folder Sync Progress**: Get completion percentage and items remaining for a specific folder.
- **FR-2: Device Sync Status**: Get completion percentage and items remaining for a specific device.

## Non-Functional Requirements
- **NFR-1: Token Optimization**: Provide a concise status summary.
- **NFR-2: Performance**: Minimize API calls to retrieve comprehensive status.

## Acceptance Criteria
- [ ] `get_sync_status(target="folder")` returns accurate completion and item remaining counts.
- [ ] `get_sync_status(target="device")` returns accurate completion and item remaining counts.
- [ ] Docker integration tests verify status reporting against a live instance.
