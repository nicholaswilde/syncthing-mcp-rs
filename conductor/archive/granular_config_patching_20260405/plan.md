# Implementation Plan: Granular Configuration Patching

## Phase 1: API Client Implementation [checkpoint: 94a71c3]
- [x] Implement `patch_folder_config()` in `src/api/client.rs`. [193cdfa]
- [x] Implement `patch_device_config()` in `src/api/client.rs`. [74af92d]
- [x] Add generic `patch_config` method to support future patching needs. [f9640f4]

## Phase 2: MCP Tools Implementation [checkpoint: 20d18f7]
- [x] Create new tool `patch_instance_config`. [5e9b448]
- [x] Integrate with `config_diff` logic to provide previews before applying changes. [89abc6f]
- [x] Register tools in `src/tools/mod.rs`. [396c9e5]

## Phase 3: Verification
- [x] Write unit and integration tests. [396c9e5]
- [x] Update documentation. [396c9e5]
