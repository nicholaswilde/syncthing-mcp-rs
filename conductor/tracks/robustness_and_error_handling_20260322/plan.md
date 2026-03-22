# Implementation Plan: Robustness & Error Handling

## Phase 1: Error Refactoring
- [x] Add `tokio-retry` to `Cargo.toml`. `479ee91`
- [ ] Update `src/error.rs` with more granular error variants.
- [ ] Implement a helper to map `SyncThingError` to MCP `ResponseError`.

## Phase 2: Retry Logic
- [ ] Wrap `SyncThingClient` methods with a retry wrapper.
- [ ] Configure retry policies (max attempts, backoff strategy).

## Phase 3: Validation
- [ ] Add unit tests for retry logic (using `wiremock` to simulate failures).
- [ ] Verify error responses via MCP Inspector.
