# CSharpX Language Specification 0.1 (DRAFT)

Normative keywords MUST, SHOULD, MAY follow RFC 2119 usage. **This is a proposed language contract, not a completed implementation.**

A conforming compiler MUST reject unsupported language features rather than silently translate them with altered semantics. Profiles MAY reject programs requiring unavailable resources, but MUST NOT silently change reference identity, exception ordering, or cleanup semantics.

## Profiles

- `desktop-full`: full managed graph and exception facilities.
- `embedded-managed`: bounded allocators, deterministic configuration and optional cycle collection.
- `bare-metal`: `no_std`, no mandatory allocator, optional fixed pools; unsupported allocation paths are compile-time errors.

## Compatibility

CSharpX resembles C# but is not a claim of bytecode, CLR, .NET library, or full C# language compatibility. All deviations require explicit specification and diagnostics.
