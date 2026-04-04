# Research & Tool Consolidation Strategy - Token Usage Optimization

## Current State Analysis
The SyncThing MCP server currently has 28 tools. Many tools return similar information or perform related actions, leading to a large tool list that consumes tokens during discovery. Additionally, several tools return verbose text or full JSON objects, which can be token-intensive.

### Key Issues:
1.  **Tool List Bloat**: Discovery of 28 tools consumes significant tokens.
2.  **Verbose Responses**: Text formatting and large JSON objects (e.g., `get_folder`) include unnecessary fields.
3.  **Redundant Calls**: Information like "health" or "stats" is spread across multiple tools.

## Consolidation Strategy
Consolidate related tools into "super-tools" using an `action` or `aspect` parameter. This reduces the total tool count and streamlines discovery.

### Proposed Super-Tools:
1.  `manage_system`: Consolidates `list_instances`, `get_instance_health`, `get_system_status`, `get_system_connections`, `get_system_log`, `get_global_dashboard`, `maintain_system`, `analyze_error`.
2.  `manage_folders`: Already exists, but will absorb `get_folder_statistics`, `manage_ignores`, `configure_sharing`.
3.  `manage_devices`: Already exists, but will absorb `get_device_statistics`.
4.  `manage_conflicts`: Consolidates `list_conflicts`, `resolve_conflict`, `delete_conflict`, `diff_conflicts`, `preview_conflict_resolution`.
5.  `manage_bandwidth`: Consolidates `set_bandwidth_limits`, `set_performance_profile`, `get_bandwidth_status`.
6.  `manage_config`: Consolidates `replicate_config`, `diff_instance_configs`, `merge_instance_configs`.

## Optimization Utilities
Implement utilities to reduce response size:

### 1. Field Aliasing
Map long SyncThing field names to shorter aliases.
Example: `in_sync_bytes` -> `isb`, `global_bytes` -> `gb`.

### 2. Content Filtering
Allow users to specify which fields they want to receive. Default to a "minimal" set for broad queries.

### 3. Response Truncation
Limit the number of items in lists (e.g., `get_system_log`, `browse_folder`) and provide pagination where appropriate.

## Targeted Token Reduction: 30%+
By reducing tool count from 28 to ~10 and optimizing response payloads, we aim for a >30% reduction in total token usage per session.
