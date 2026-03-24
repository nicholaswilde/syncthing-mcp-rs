# Track Specification: Instance Management Tools (instance_management_20260322)

## Overview
Develop a suite of tools that provide a high-level overview and health check of all configured SyncThing instances. This is essential for managing complex, multi-instance setups.

## Scope
- Implement a `list_instances` tool to show all configured instances, their IDs, URLs, and current connection status.
- Develop a `get_instance_health` tool that checks API connectivity, basic system status, and pending errors.
- Support "batch" operations to check health across all instances simultaneously.
- Provide a summary of active connections, synchronization progress, and resource usage for each instance.

## Success Criteria
- [ ] `list_instances` correctly identifies and displays all configured instances from `config.toml`.
- [ ] `get_instance_health` successfully reports connectivity and error states.
- [ ] Multi-instance setups are handled gracefully with clear per-instance status reports.
- [ ] Timeouts and connection failures are accurately reported without crashing the server.
- [ ] Unit tests verify health check logic and error reporting.
