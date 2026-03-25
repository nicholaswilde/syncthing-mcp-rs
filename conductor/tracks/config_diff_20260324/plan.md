# Implementation Plan - Implement Configuration Diffing and Merging Tool

## Phase 1: Core Diffing Logic
- [x] Task: Define ConfigDiff and ConfigPatch structures (6ead759)
    - [x] Create data models for representing differences and patches in `src/tools/config_diff.rs`
- [x] Task: Implement diffing function (6ead759)
    - [x] Write tests for diffing two `Config` objects
    - [x] Implement `calculate_diff(base: &Config, head: &Config) -> ConfigDiff`
- [x] Task: Implement patching function (6ead759)
    - [x] Write tests for applying a `ConfigPatch` to a `Config`
    - [x] Implement `apply_patch(config: &mut Config, patch: &ConfigPatch) -> Result<()>`
- [x] Task: Conductor - User Manual Verification 'Core Diffing Logic' (Protocol in workflow.md) (a8a0f3c)

## Phase 2: MCP Tool Integration
- [x] Task: Implement `diff_instance_configs` tool handler (e8a017d)
    - [x] Write tests for the tool handler in `src/tools/config_tests.rs`
    - [x] Implement the `diff_instance_configs` function that fetches configs from two instances and returns a diff
- [x] Task: Register the tool in `ToolRegistry` (e8a017d)
    - [x] Add the new tool to `src/tools/mod.rs`
- [x] Task: Implement configuration merging tool (e8a017d)
    - [x] Write tests for merging configurations
    - [x] Create `merge_instance_configs` tool to apply a diff/patch
- [x] Task: Conductor - User Manual Verification 'MCP Tool Integration' (Protocol in workflow.md) (e8a017d)

## Phase 3: Validation and Refinement
- [x] Task: Docker integration tests (614f4a9)
    - [x] Add new tests to `tests/docker_integration_test.rs` to verify cross-instance diffing and merging
- [ ] Task: Documentation and Quality Gates
    - [ ] Update `README.md` with the new tools
    - [ ] Finalize code documentation and verify coverage (>90%)
- [ ] Task: Conductor - User Manual Verification 'Validation and Refinement' (Protocol in workflow.md)
