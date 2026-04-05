# Specification: File & Sync Diagnostics

## Objective
Implement endpoints for granular file and sync diagnostics to help debug synchronization issues.

## Requirements
- Add `get_file_info` using `GET /rest/db/file` to return detailed metadata about a specific file or directory within a folder.
- Add `get_folder_needs` using `GET /rest/db/needs` to return a list of files needed to bring a folder up to date.
- Create corresponding MCP tools to expose these diagnostic functions.
- Add comprehensive tests.