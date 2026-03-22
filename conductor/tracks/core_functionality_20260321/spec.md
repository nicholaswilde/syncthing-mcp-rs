# Track Specification: Build core SyncThing MCP server functionality (core_functionality_20260321)

## Overview
Implement the foundational structure of the SyncThing MCP server in Rust, enabling basic interactions with the SyncThing REST API through the Model Context Protocol.

## Scope
- Scaffold the Rust project structure.
- Implement configuration loading for multiple SyncThing instances.
- Develop the core MCP server with `stdio` transport.
- Implement basic `manage_folders` and `get_system_stats` tools.
- Set up automated testing and linting.

## Success Criteria
- [ ] Successful `cargo build` with no errors.
- [ ] `config.toml` correctly parses multiple instances.
- [ ] `mcp-inspector` verifies the `get_system_stats` tool.
- [ ] `manage_folders` tool correctly lists folders from a mock SyncThing API.
- [ ] 100% pass rate for unit tests.
