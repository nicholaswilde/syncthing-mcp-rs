# Track Specification: Sharing Orchestration (configure_sharing_20260321)

## Overview
Implement the `configure_sharing` tool to share or unshare folders with specific devices.

## Functional Requirements
- **FR-1: Share Folder**: Add a device to a folder's share list.
- **FR-2: Unshare Folder**: Remove a device from a folder's share list.
- **FR-3: Sync Configuration**: Ensure changes are correctly applied to the Syncthing configuration.

## Non-Functional Requirements
- **NFR-1: Token Optimization**: Summarize folder/device association in the output.
- **NFR-2: Robustness**: Handle cases where the folder or device does not exist.

## Acceptance Criteria
- [ ] `configure_sharing(action="share")` adds a device to a folder.
- [ ] `configure_sharing(action="unshare")` removes a device from a folder.
- [ ] Docker integration tests verify all sharing operations against a live Syncthing instance.
