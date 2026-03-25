# Track Specification: Global Dashboard Tool (global_dashboard_20260324)

## Overview
Develop a tool to aggregate statistics and health reports across all SyncThing instances into a high-level JSON summary.

## Scope
- Aggregate transfer rates across all monitored SyncThing instances.
- Collect sync data (completion, data remaining) for all folders and devices.
- Generate a high-level JSON summary of network health.
- Provide a unified report on instance status (online/offline, errors, uptime).
- Support historical data collection for trend analysis.

## Success Criteria
- [ ] Users can get an aggregated view of their entire SyncThing network.
- [ ] Transfer rates and sync progress are clearly reported across all instances.
- [ ] The global JSON summary is easy to parse for external dashboards.
- [ ] Automated tests verify data aggregation and report generation.
