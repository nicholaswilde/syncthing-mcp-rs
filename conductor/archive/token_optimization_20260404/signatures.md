# Super-Tool Signatures & Optimization Parameters

## 1. `manage_system`
Consolidates all system-level status, health, and maintenance tools.

### Parameters:
- `action` (required): `status`, `health`, `connections`, `log`, `dashboard`, `rescan`, `restart`, `shutdown`, `clear_errors`, `analyze_error`.
- `instance` (optional): Target instance name/index.
- `error_message` (required for `analyze_error`): Technical error message to analyze.
- `fields` (optional): List of fields to include in JSON output (e.g., `uptime,version,my_id`).
- `limit` (optional): Max log entries to return for `log` action. Default: 50.

---

## 2. `manage_folders`
Consolidates all folder management and status tools.

### Parameters:
- `action` (required): `list`, `get`, `stats`, `pending`, `reject_pending`, `revert`, `share`, `unshare`, `ignores_get`, `ignores_set`, `ignores_append`.
- `folder_id` (required for most actions).
- `device_id` (required for `share`/`unshare`).
- `patterns` (required for `ignores_set`/`ignores_append`): List of ignore patterns.
- `fields` (optional): List of fields to include in JSON output.
- `shorten` (optional): If true, use short aliases for fields (e.g., `isb` for `in_sync_bytes`). Default: true.

---

## 3. `manage_devices`
Consolidates all device management and status tools.

### Parameters:
- `action` (required): `list`, `stats`, `add`, `remove`, `pause`, `resume`, `discover`, `approve`, `validate`.
- `device_id` (required for most actions).
- `name` (optional): Friendly name for the device.
- `fields` (optional): List of fields to include in JSON output.
- `shorten` (optional): If true, use short aliases for fields. Default: true.

---

## 4. `manage_conflicts`
Consolidates all conflict management tools.

### Parameters:
- `action` (required): `list`, `resolve`, `delete`, `diff`, `preview`.
- `folder_id` (required for `list`).
- `conflict_path` (required for `resolve`, `delete`, `diff`, `preview`).
- `resolution` (required for `resolve`/`preview`): `keep_original` or `keep_conflict`.
- `backup` (optional): If true, move deleted files to trash. Default: true.
- `limit` (optional): Max conflicts to return for `list`. Default: 20.

---

## 5. `manage_bandwidth`
Consolidates all bandwidth and performance profile tools.

### Parameters:
- `action` (required): `get_status`, `set_limits`, `set_profile`.
- `max_recv_kbps` (optional for `set_limits`).
- `max_send_kbps` (optional for `set_limits`).
- `profile_name` (required for `set_profile`).

---

## 6. `manage_config`
Consolidates all multi-instance configuration tools.

### Parameters:
- `action` (required): `replicate`, `diff`, `merge`.
- `source` (optional): Source instance name/index.
- `destination` (required): Destination instance name/index.
- `dry_run` (optional): If true, preview changes only. Default: false.
- `folders` (optional): Specific folder IDs to replicate.
- `devices` (optional): Specific device IDs to replicate.
