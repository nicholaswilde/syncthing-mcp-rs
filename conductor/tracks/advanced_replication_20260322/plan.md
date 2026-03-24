# Implementation Plan: Advanced Config Replication (advanced_replication_20260322)

## Phase 1: Logic & Validation [checkpoint: 585c649]
- [x] Task: Refactor the replication logic to support a "dry-run" mode. 7ae15d6
- [x] Task: Implement input validation for selective folder/device filters. def63bb
- [x] Task: Create a diff generator for SyncThing configuration changes. 1e7aacb


## Phase 2: Tool Enhancements [checkpoint: 25a8268]
- [x] Task: Update the `replicate_config` MCP tool with new optional parameters. 4bd2534
- [x] Task: Implement the folder-level replication logic. 5c0a74c
- [x] Task: Implement the device-level replication logic. 5c0a74c

## Phase 3: Reporting & Safety [checkpoint: e3f3e78]
- [x] Task: Format the "dry-run" output to be clear and readable. 1e7aacb
- [x] Task: Add warnings for potential configuration conflicts or data loss risks. c798c87
- [ ] Task: Implement backup-on-change for target instances before replication.

## Phase 4: Validation
- [ ] Task: Unit tests for selective replication logic with mock configurations.
- [ ] Task: Integration tests verifying `dry_run` behavior.
- [ ] Task: Test complex replication scenarios involving multiple folders and devices.
