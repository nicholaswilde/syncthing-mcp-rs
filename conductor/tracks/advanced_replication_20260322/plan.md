# Implementation Plan: Advanced Config Replication (advanced_replication_20260322)

## Phase 1: Logic & Validation [checkpoint: 585c649]
- [x] Task: Refactor the replication logic to support a "dry-run" mode. 7ae15d6
- [x] Task: Implement input validation for selective folder/device filters. def63bb
- [x] Task: Create a diff generator for SyncThing configuration changes. 1e7aacb


## Phase 2: Tool Enhancements
- [x] Task: Update the `replicate_config` MCP tool with new optional parameters. 4bd2534
- [ ] Task: Implement the folder-level replication logic.
- [ ] Task: Implement the device-level replication logic.

## Phase 3: Reporting & Safety
- [ ] Task: Format the "dry-run" output to be clear and readable.
- [ ] Task: Add warnings for potential configuration conflicts or data loss risks.
- [ ] Task: Implement backup-on-change for target instances before replication.

## Phase 4: Validation
- [ ] Task: Unit tests for selective replication logic with mock configurations.
- [ ] Task: Integration tests verifying `dry_run` behavior.
- [ ] Task: Test complex replication scenarios involving multiple folders and devices.
