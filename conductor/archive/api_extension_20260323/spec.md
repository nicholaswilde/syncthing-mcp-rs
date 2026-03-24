# Specification - API Extension and Review

## Objective
The goal of this track is to identify and implement missing Syncthing REST API endpoints that would improve the utility and completeness of the MCP server.

## Current Implementation

### Implemented Endpoints
- `GET /rest/system/status`
- `GET /rest/system/version`
- `GET /rest/config`
- `PUT /rest/config`
- `GET /rest/config/folders`
- `POST /rest/config/folders`
- `GET /rest/config/folders/{id}`
- `PATCH /rest/config/folders/{id}`
- `GET /rest/db/ignores`
- `POST /rest/db/ignores`
- `GET /rest/config/devices`
- `POST /rest/config/devices`
- `DELETE /rest/config/devices/{id}`
- `PATCH /rest/config/devices/{id}`
- `GET /rest/db/status`
- `GET /rest/db/completion`
- `POST /rest/db/scan`
- `POST /rest/system/restart`
- `POST /rest/system/error/clear`
- `GET /rest/events`
- `GET /rest/db/browse`
- `GET /rest/cluster/pending/devices`
- `DELETE /rest/cluster/pending/devices`

## Identified Missing Endpoints

### High Priority
These endpoints provide significant functional value for management and monitoring.

- `GET /rest/system/connections`: Returns current connections to other devices. Essential for real-time monitoring.
- `GET /rest/system/log`: Returns the recent log entries. Critical for troubleshooting.
- `GET /rest/stats/device`: Connection statistics for each device (e.g., last seen).
- `GET /rest/stats/folder`: Statistics for each folder (e.g., last scan time).
- `GET /rest/db/file`: Returns metadata about a specific file.
- `POST /rest/db/revert`: Reverts local changes in a "Send Only" folder. Important for recovery.
- `GET /rest/cluster/pending/folders`: Lists folders that other devices are trying to share.
- `DELETE /rest/cluster/pending/folders`: Rejects a pending folder share.

### Medium Priority
These endpoints add more complete management coverage but are less critical for daily use.

- `POST /rest/system/shutdown`: Shuts down the Syncthing instance.
- `GET /rest/svc/deviceid`: Validates and canonicalizes device IDs.
- `GET /rest/config/options`: Provides granular access to system options.
- `GET /rest/config/gui`: Provides granular access to GUI settings.

## Implementation Requirements
- Each new endpoint should be implemented in `SyncThingClient`.
- Corresponding models should be added to `src/api/models.rs`.
- New tools should be added to `src/tools/` and registered in `src/tools/mod.rs`.
- Unit tests should be added for each new client method and tool.
