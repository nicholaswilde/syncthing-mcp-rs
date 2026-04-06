# Specification: Configuration & Error Management

## Objective
Implement endpoints to verify configuration consistency and retrieve explicit system GUI errors.

## Requirements
- Add `is_config_insync` using `GET /rest/system/config/insync` to verify if the running config matches the on-disk config.
- Add `get_errors` using `GET /rest/system/error` to retrieve the current list of active system GUI errors.
- Create corresponding MCP tools for these functions.
- Add comprehensive tests.