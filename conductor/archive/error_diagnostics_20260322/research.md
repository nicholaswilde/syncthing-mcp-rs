# Research: SyncThing REST API Error Messages & Behaviors

## Common Error Patterns

| Error Type | Status Code | Typical Message | Behavior |
|------------|-------------|-----------------|----------|
| **Unauthorized** | 401 | `Unauthorized` | Missing or invalid `X-API-Key` or `Authorization` header. |
| **Forbidden** | 403 | `CSRF Error` | Missing or invalid CSRF token (common when not using API key). |
| **Not Found** | 404 | `404 page not found` | Invalid endpoint or resource ID (e.g., folder/device ID). |
| **Access Denied** | 403 | `access denied` | Occurs when calling restricted debugging endpoints (e.g., `/rest/debug/profile`). |
| **Internal Server Error** | 500 | Plain text message | Generic failure. E.g., `folder "default" does not exist`, `invalid device ID`. |
| **Conflict** | 409 | Depends on endpoint | Often related to configuration updates that conflict with current state. |
| **Bad Request** | 400 | JSON/Plain text | Invalid JSON body or missing required query parameters. |

## Detailed Error Messages (Observed in Forum/Docs)

- `folder "ID" not found`: Occurs in `/rest/db/scan` or `/rest/db/status`.
- `device "ID" not found`: Occurs in `/rest/system/resume` or `/rest/system/pause`.
- `scan of a path that no longer exists`: Returned in `/rest/db/scan` for missing paths.
- `invalid device ID`: Returned when a malformed device ID is provided.
- `out of disk space`: Can be observed in log entries or specific status endpoints.
- `connection refused`: Network level, SyncThing instance not running or port blocked.
- `context deadline exceeded`: Timeout, often when the server is overloaded.

## Diagnostic Advice (Target for Engine)

- **401/403 (Auth)**: Verify API key in configuration. Check if the key has expired or changed.
- **404 (Not Found)**: List all folders/devices to confirm the ID exists.
- **500 (ID not found)**: Similar to 404, but specifically for API endpoints that don't return 404 for missing resources.
- **CSRF Error**: Ensure the API key is used, as it bypasses CSRF checks.
- **Connection Refused**: Check if SyncThing process is running. Verify the URL and port.
