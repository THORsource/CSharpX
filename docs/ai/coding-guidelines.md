# AI coding guidelines

1. Write `.csx` code, not Rust.
2. Use C#-style classes, exceptions, `using` and interfaces according to the versioned specification.
3. Do not introduce `unsafe` unless the task explicitly requires raw memory or untrusted native calls.
4. Use safe audited wrappers for native hardware access where available.
5. Honor `[NoAllocation]`, `[NoGC]`, `[NoBlocking]`, `[NoThrow]` transitively.
6. Check target capabilities before using heap, threads, filesystem, or networking.
7. Do not invent standard-library symbols: use the published API index.
8. Validate with `csxc check` and tests; currently only the documented prototype subset is implemented.
9. Explain unsupported features instead of silently substituting semantics.
