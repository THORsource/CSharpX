# Conformance test matrix (planned)

- Syntax: tokens, parsing, diagnostics and recovery
- Types: overloads, generics, nullability and conversions
- Objects: inheritance, dynamic dispatch, identity and casts
- Ownership: alias/escape, sharing, object lifetime and cycles
- Exceptions: typed catch, filters, rethrow, finally, using and async
- Interop: C ABI, C++ shim, ownership, errors, native exceptions
- Embedded: no_std, no heap, interrupts, bounded memory and multicore
- Safety: generated safe code contains no unsafe; trusted boundaries audited
- IDE: navigation, completion, diagnostics, rename, templates, build and debug

The Rust unit tests cover only the minimal compiler prototype.
