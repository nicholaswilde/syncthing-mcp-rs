# Track Specification: Device Management (manage_devices_20260321)

## Overview
Implement the `manage_devices` tool to list, add, remove, pause, and resume Syncthing devices.

## Functional Requirements
- **FR-1: List Devices**: Retrieve a list of all configured devices.
- **FR-2: Add Device**: Add a new device by its Device ID and Name.
- **FR-3: Remove Device**: Remove an existing device.
- **FR-4: Pause/Resume Device**: Control the connection state of a device.

## Non-Functional Requirements
- **NFR-1: Token Optimization**: Summarize output for list operations to minimize token usage.
- **NFR-2: Error Handling**: Provide clear error messages for invalid Device IDs or API failures.

## Acceptance Criteria
- [ ] `manage_devices(action="list")` returns a summarized list of devices.
- [ ] `manage_devices(action="add")` successfully adds a device.
- [ ] `manage_devices(action="remove")` successfully removes a device.
- [ ] `manage_devices(action="pause")` and `resume` work as expected.
- [ ] Docker integration tests verify all actions against a live Syncthing instance.
