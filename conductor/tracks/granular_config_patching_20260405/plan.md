# Implementation Plan: Granular Configuration Patching

## Phase 1: API Client Implementation [checkpoint: 94a71c3]
- [x] Implement `patch_folder_config()` in `src/api/client.rs`. [193cdfa]
- [x] Implement `patch_device_config()` in `src/api/client.rs`. [74af92d]
- [x] Add generic `patch_config` method to support future patching needs. [f9640f4]

## Phase 2: MCP Tools Implementation
- [~] Create new tool `patch_instance_config`.
- [ ] Integrate with `config_diff` logic to provide previews before applying changes.
- [ ] Register tools in `src/tools/mod.rs`.

## Phase 3: Verification
- [ ] Write unit and integration tests.
- [ ] Update documentation.
