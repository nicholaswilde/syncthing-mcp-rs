# Implementation Plan - API Extension and Review

## Phase 1: High-Priority Endpoints [checkpoint: 39f1903]
Focus on endpoints that enhance the monitoring and troubleshooting capabilities of the MCP server.

### Tasks
- [x] Implement `GET /rest/system/connections` (321d1c4)
  - Define `ConnectionStatus` model.
  - Implement `get_connections` in `SyncThingClient`.
  - Add `get_system_connections` tool to `system.rs`.
- [x] Implement `GET /rest/system/log` (bdc574f)
  - Define `LogEntry` model.
  - Implement `get_system_log` in `SyncThingClient`.
  - Add `get_system_log` tool to `system.rs`.
- [x] Implement `GET /rest/stats/device` and `GET /rest/stats/folder` (94c1300)
  - Define relevant models.
  - Implement client methods.
  - Add tools to `devices.rs` and `folders.rs`.
- [x] Implement `GET /rest/cluster/pending/folders` (88e16f3)
  - Define `PendingFolder` model.
  - Implement `get_pending_folders` and `remove_pending_folder` in `SyncThingClient`.
  - Add `manage_pending_folders` tool to `folders.rs`.

## Phase 2: Configuration and Utility Endpoints [checkpoint: 031efcc]
Focus on providing more granular management capabilities.

### Tasks
- [x] Implement `POST /rest/db/revert` (53d2f33)
  - Implement `revert_folder` in `SyncThingClient`.
  - Add to `folders.rs`.
- [x] Implement `POST /rest/system/shutdown` (0f6c144)
  - Implement `shutdown` in `SyncThingClient`.
  - Add to `system.rs`.
- [x] Implement `GET /rest/svc/deviceid` (8418cb4)
  - Implement `validate_device_id` in `SyncThingClient`.
  - Add to `devices.rs`.

## Phase 3: Review and Refactor
- [ ] Review all tool descriptions for clarity and consistency.
- [ ] Ensure all tools have comprehensive unit tests.
- [ ] Validate implementation against real Syncthing instances (if possible).
