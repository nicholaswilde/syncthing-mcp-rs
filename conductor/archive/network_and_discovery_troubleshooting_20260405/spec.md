# Specification: Network & Discovery Troubleshooting

## Objective
Implement endpoints for network and global discovery status to help diagnose connectivity issues between devices.

## Requirements
- Add `get_discovery_status` using `GET /rest/system/discovery` to return local and global discovery status.
- Create corresponding MCP tools to expose these diagnostic functions.
- Add comprehensive tests.