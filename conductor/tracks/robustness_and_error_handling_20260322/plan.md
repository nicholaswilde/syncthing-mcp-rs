# Implementation Plan: Robustness & Error Handling

## Phase 1: Error Refactoring [checkpoint: 1924758]
- [x] Add `tokio-retry` to `Cargo.toml`. `479ee91`
- [x] Update `src/error.rs` with more granular error variants. `ad2d0b1`
- [x] Implement a helper to map `SyncThingError` to MCP `ResponseError`. `5b45c5f`

## Phase 2: Retry Logic [checkpoint: f7d6076]
- [x] Wrap `SyncThingClient` methods with a retry wrapper. `1abd821`
- [x] Configure retry policies (max attempts, backoff strategy). `ec72037`

## Phase 3: Validation [checkpoint: e738ddd]
- [x] Add unit tests for retry logic (using `wiremock` to simulate failures). `069f887`
- [x] Verify error responses via MCP Inspector. `697ce86`
