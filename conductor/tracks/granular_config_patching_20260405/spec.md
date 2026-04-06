# Specification: Granular Configuration Patching

## Objective
Enable targeted configuration updates for folders and devices without requiring a full configuration overwrite.

## Requirements
- Implement `PATCH` support for `/rest/config/folders/{id}`.
- Implement `PATCH` support for `/rest/config/devices/{id}`.
- Create a unified `patch_config` MCP tool that handles both folders and devices.
- Include validation and dry-run (preview) capabilities.
- Comprehensive unit and integration tests.
