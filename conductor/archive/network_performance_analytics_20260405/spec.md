# Specification: Network Performance Analytics

## Objective
Provide deeper insights into the peer network, including connection types, granular transfer rates, and troubleshooting data.

## Requirements
- Enhance `get_system_connections` to report protocol types (TCP, QUIC, Relay) and per-connection throughput.
- Add support for viewing address resolution and NAT traversal status.
- Create a troubleshooting summary for "disconnected" or "degraded" connections.
- Comprehensive unit and integration tests.
