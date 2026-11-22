# Runtime architecture (not yet implemented)

Components: core object identity and type metadata, owned/arena/pool allocators, RC/ARC, cycle tracing, exception transport, task scheduling, and native trust boundary. Implement safe Rust by default. Runtime internals requiring unsafe are isolated, documented and audited. No collector is shipped in this starter.
