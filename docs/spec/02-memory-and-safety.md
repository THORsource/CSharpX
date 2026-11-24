# Memory, ownership and safety (DRAFT)

## Ownership inference

Compiler performs escape, alias, mutation, interprocedural effect and lifetime analyses. Allocation strategies include stack, owned, arena, fixed pool, shared RC/ARC, and cycle-managed graphs. Optimizations MUST preserve class identity and observable destruction/disposal behavior.

## Cycle collection

Cycles require tracing or equivalent collection; RC alone is insufficient. The runtime defines roots, tracing metadata, safe points, thread synchronization, and behavior under pool exhaustion. Real-time profiles must state worst-case bounds or reject collection in restricted scopes.

## Unsafe rule

For generated application modules without source `unsafe`, the backend MUST emit no Rust `unsafe` constructs and SHOULD emit `#![forbid(unsafe_code)]`. Explicit CSharpX `unsafe` may produce Rust `unsafe` only for corresponding operations. Runtime/HAL and native code are separate audited trust boundaries. Safe bindings require audited preconditions; a user annotation alone cannot confer trust.

## Effects

`Allocate`, `Collect`, `Block`, `Throw`, `IO`, `Sync`, `UnsafeHardware` are propagated through call graphs. `[NoAllocation]`, `[NoGC]`, `[NoBlocking]`, `[NoThrow]` trigger transitive checks.

The scalar compiler supports int* and bool* to local variables, address-of, dereference and assignment through a pointer inside unsafe blocks. The programmer is responsible for pointee lifetime. Only generated pointer reads and writes receive a local exemption from deny(unsafe_code).
