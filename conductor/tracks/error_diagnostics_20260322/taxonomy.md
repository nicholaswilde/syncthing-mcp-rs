# Error Taxonomy: SyncThing MCP

This document defines the structured taxonomy for mapping technical SyncThing errors to actionable categories and diagnostic advice.

## Taxonomy Categories

| Category | Description | Root Causes |
|----------|-------------|-------------|
| **Network** | Connectivity issues between the MCP server and SyncThing. | Connection refused, timeout, DNS resolution failure. |
| **Permission** | Authorization and authentication failures. | Missing/invalid API key, CSRF error, restricted endpoint. |
| **Configuration** | Invalid parameters, missing resources, or state conflicts. | Invalid folder/device ID, malformed JSON, conflicting config. |
| **Resource** | System-level resource exhaustion or path issues. | Out of disk space, missing file path, database locked. |
| **Internal** | Unexpected failures within the MCP server itself. | Logic errors, unhandled exceptions. |

## Mapping Table (Technical -> Taxonomy)

| Technical Pattern | Category | Contextual Advice |
|-------------------|----------|-------------------|
| `401 Unauthorized` | **Permission** | API key is missing or invalid. Check your configuration. |
| `403 Forbidden / CSRF` | **Permission** | CSRF protection is active. Ensure you're using an API key, as it bypasses CSRF checks. |
| `Connection refused` | **Network** | SyncThing instance is not running or is listening on a different port. |
| `Timeout / Context deadline exceeded` | **Network** | The request took too long. Check if the server is under heavy load or network is unstable. |
| `404 Not Found` | **Configuration** | The requested endpoint or resource does not exist. Verify the ID. |
| `folder ".*" not found` | **Configuration** | Specified folder ID is incorrect. List folders to see valid IDs. |
| `device ".*" not found` | **Configuration** | Specified device ID is incorrect. List devices to see valid IDs. |
| `out of disk space` | **Resource** | SyncThing cannot write data. Check disk space on the target machine. |
| `scan of a path that no longer exists` | **Configuration** | The folder path was deleted outside of SyncThing. Trigger a rescan to acknowledge deletion. |
| `invalid device ID` | **Configuration** | The provided device ID does not follow the expected format. |
| `database locked` | **Resource** | Another instance of SyncThing might be running or the DB is corrupted. |

## Actionable Next Steps (AI Guidance)

- **Network Error**: Suggest checking the URL and port. Propose a "ping" check if possible.
- **Permission Error**: Suggest verifying the environment variables or configuration file for the API key.
- **Configuration Error**: Suggest listing the relevant resources (folders, devices) to confirm existence and current status.
- **Resource Error**: Suggest system-level checks for disk space or file permissions.
