# Specification: Developer Experience & Security

## Goal
Improve the efficiency and security profile of the SyncThing MCP server.

## Requirements
- Optimize binary size by evaluating dependencies (e.g., consider replacing `reqwest` if only basic JSON features are used).
- Implement more secure ways to store and pass API keys, such as OS-level secret stores.
- Expand support for environment variables in the configuration system.
- Minimize the use of unsafe code in critical paths.

## Success Criteria
- [ ] Reduced final binary size by at least 10%.
- [ ] API keys can be passed through more secure channels than plain text config files.
- [ ] Build process for all target architectures remains stable.
