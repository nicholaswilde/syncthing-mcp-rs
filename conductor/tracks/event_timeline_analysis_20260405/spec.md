# Specification: Event Timeline Analysis

## Objective
Implement historical event windowing and analysis for retrospective debugging and monitoring.

## Requirements
- Add a tool to fetch and window events based on a time duration (e.g., "last 10 minutes").
- Implement semantic analysis of event sequences (e.g., "detect rapid flap of device connections").
- Create a `get_event_timeline` tool that returns a human-readable history of instance activity.
- Comprehensive unit and integration tests.
