# Specification: Code Quality

## Goal
Improve codebase maintainability and clarify the server's capabilities.

## Requirements
- Move as much logic as possible from `src/main.rs` into a new `run()` function in `src/lib.rs`.
- Implement automated generation of documentation for the MCP tools.
- Refine tool descriptions and argument metadata in the tool registry.
- Ensure all public functions are documented using standard Rust doc comments.

## Success Criteria
- [ ] `src/main.rs` contains minimal code (primarily just calling `lib::run()`).
- [ ] New unit tests cover the server's initialization and startup logic.
- [ ] Documentation for all tools is generated and easily accessible.
