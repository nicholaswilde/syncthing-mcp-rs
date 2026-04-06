# Specification: System Maintenance & Lifecycle

## Objective
Implement endpoints to manage the Syncthing application lifecycle, including upgrades and lightweight ping checks.

## Requirements
- Add `check_upgrade` using `GET /rest/system/upgrade` to verify if a newer version is available.
- Add `perform_upgrade` using `POST /rest/system/upgrade` to trigger a software update.
- Add `ping` using `GET /rest/system/ping` to test API responsiveness lightweightly.
- Create corresponding MCP tools for these functions.
- Add comprehensive tests.