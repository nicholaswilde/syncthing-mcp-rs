# Specification: Advanced Sync Operations

## Objective
Implement tools for granular file-level synchronization control and per-device completion monitoring.

## Requirements
- Add `set_file_priority` using `POST /rest/db/prio` to move specific items to the top of the download queue.
- Add `get_device_completion` using `GET /rest/db/completion` to retrieve precise sync status for a folder on a specific device.
- Integrate these into existing "super-tools" like `inspect_folder` where appropriate.
- Comprehensive unit and integration tests.
