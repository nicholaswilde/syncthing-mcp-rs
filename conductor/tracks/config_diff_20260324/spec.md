# Specification - Implement Configuration Diffing and Merging Tool

## Overview
This track focuses on implementing a robust configuration diffing and merging tool for the SyncThing MCP server. This tool will allow users to compare configurations between two SyncThing instances, identify differences, and apply selective changes from one to another.

## Functional Requirements
- **Diff Generation**: Generate a detailed difference report between two `Config` objects.
- **Selective Merge**: Apply selected differences from a source configuration to a target configuration.
- **Safety**: Ensure that merging configurations doesn't lead to invalid states.
- **MCP Tool**: Expose the diffing and merging capability as a high-level MCP tool `diff_instance_configs`.
- **Conflict Handling**: Properly handle cases where both configurations have modified the same field (though primarily unidirectional merge is expected).

## Technical Requirements
- **Rust Implementation**: Core logic in `src/tools/config_diff.rs`.
- **Serde Integration**: Leverage Serde for deep comparison and JSON-based diff representation.
- **Error Handling**: Use the existing `Error` and `Diagnostic` systems for informative error messages.
- **Testing**:
  - Unit tests for diff calculation and merge logic.
  - Integration tests using Docker to verify real-world behavior between two SyncThing instances.

## Design Decisions
- **JSON Patch**: Consider using a standard JSON Patch format (RFC 6902) or a simplified internal representation for diffs.
- **Instance Configuration**: The tool should work with instance names as defined in the `AppConfig`.
