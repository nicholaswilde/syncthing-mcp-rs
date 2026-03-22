# Specification: Robustness & Error Handling

## Goal
Make the SyncThing MCP server resilient to network failures and provide meaningful error messages to the LLM.

## Requirements
- Use `tokio-retry` or similar for exponential backoff on transient network errors (e.g., connection timed out).
- Define a set of structured error types in `src/error.rs`.
- Map HTTP status codes (401, 403, 404, 500) to specific MCP error responses.
- Ensure that sensitive information (like API keys) is never included in error logs.

## Success Criteria
- [ ] Intermittent network failures are automatically retried.
- [ ] Tools return descriptive errors instead of generic "Internal error" when possible.
- [ ] No regression in existing test coverage.
