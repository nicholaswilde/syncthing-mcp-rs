# Specification: Token Usage Optimization Track

## Overview
This track aims to optimize the SyncThing MCP server's token usage, fulfilling Core Goal #2: "Provide high-level, functional tools... rather than exposing raw, granular API endpoints, optimizing for LLM token usage."

## Functional Requirements
- **Tool Consolidation:** Review and merge granular tools into higher-level "super-tools" to reduce the number of tool calls and overhead.
    - `inspect_folder`: Consolidates sync status, conflicts, and folder statistics.
    - `get_instance_overview`: Consolidates system status, peer connections, and instance health.
    - `inspect_device`: Consolidates device sync status and statistics.
    - `batch_manage_folders`: Enables bulk actions (rescan, revert) on multiple folders.
    - `summarize_conflicts`: Provides an actionable summary of conflicts across all folders.
- **Content Filtering:** Implement filters to remove redundant or non-essential metadata from tool responses.
- **Response Truncation:** Apply stricter defaults for list/depth limits, with user-override options.
- **Field Aliasing:** Explore using more compact field names (e.g., `id` instead of `folder_id`) where context is clear.
- **Global Application:** Apply optimizations to Folder Management, File Browsing, and System Status/Stats.

## Non-Functional Requirements
- **Performance:** Ensure optimizations do not negatively impact response latency.
- **Backwards Compatibility:** Maintain essential functionality while changing tool signatures if necessary.
- **Goal:** Achieve a 30% or greater reduction in average response size (in tokens).

## Acceptance Criteria
- [ ] At least 2 "super-tools" implemented that replace multiple granular ones.
- [ ] Average response size for `list_folders` and `browse_files` reduced by 30%+.
- [ ] Documentation updated to reflect the new, optimized toolset.
- [ ] All existing integration tests pass with the new tools.

## Out of Scope
- Rewriting the core SyncThing API (this is an MCP-layer optimization).
- Changing the transport protocol (stdio/SSE).
