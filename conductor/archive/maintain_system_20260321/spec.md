# Track Specification: Maintenance Operations (maintain_system_20260321)

## Overview
Implement the `maintain_system` tool to perform maintenance tasks on a Syncthing instance.

## Functional Requirements
- **FR-1: Force Rescan**: Trigger a rescan of a specific folder.
- **FR-2: Restart**: Trigger a restart of the Syncthing instance.
- **FR-3: Clear Errors**: Clear any existing system or folder errors.

## Non-Functional Requirements
- **NFR-1: Token Optimization**: Provide a summarized status after each maintenance operation.
- **NFR-2: Resilience**: Handle connection losses during restarts.

## Acceptance Criteria
- [ ] `maintain_system(action="force_rescan")` successfully triggers a rescan.
- [ ] `maintain_system(action="restart")` triggers a restart.
- [ ] `maintain_system(action="clear_errors")` clears errors successfully.
- [ ] Docker integration tests verify all maintenance operations against a live Syncthing instance.
