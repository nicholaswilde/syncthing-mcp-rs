# Specification: Feature Gaps

## Goal
Make the server more interactive and comprehensive by adding notifications and new tools.

## Requirements
- Subscribe to SyncThing's Event API and send MCP notifications for key events (e.g., FolderStateChanged).
- Add a tool `browse_folder` to list files and subdirectories within a synced folder.
- Add a tool `discover_devices` to list pending device requests and an action to approve them.
- Ensure event polling doesn't block the main tool execution loop.

## Success Criteria
- [ ] Notifications appear in the MCP host console during sync operations.
- [ ] Files can be listed using the browse tool.
- [ ] New devices can be onboarded entirely through MCP tools.
