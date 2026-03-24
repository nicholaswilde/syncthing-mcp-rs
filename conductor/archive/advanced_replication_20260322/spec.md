# Track Specification: Advanced Config Replication (advanced_replication_20260322)

## Overview
Enhance the existing `replicate_config` tool to support more granular control, including selective folder/device replication and a "dry-run" mode to preview changes before applying them.

## Scope
- Implement a `dry_run` flag in `replicate_config` that shows proposed changes without performing them.
- Support selective replication of specific folders by their IDs.
- Support selective replication of specific devices by their IDs.
- Add validation logic to prevent circular or inconsistent replication requests.
- Provide detailed diff-like output of the proposed configuration changes.

## Success Criteria
- [ ] `dry_run` mode accurately predicts and displays configuration changes.
- [ ] Selective replication of folders and devices functions correctly.
- [ ] Replication respects dependencies (e.g., ensuring devices exist on the target instance).
- [ ] Error messages are clear when replication is blocked by safety checks.
- [ ] Automated tests verify selective replication and `dry_run` logic.
