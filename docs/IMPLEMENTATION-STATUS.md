# Implementation status — 2026-11-22

## Executable subset
- `csxc check`, `emit-rust`, `build` for **single .csx file** (requires Rust toolchain).
- Lexer: identifiers, integer/string literals, comments, punctuation, operators.
- Parser: optional class wrapper, Main, local `int`/`string`/`var`, assignment, `Console.WriteLine`, arithmetic, `try/catch/finally`, `throw`.
- Basic static checking of names and arithmetic types.
- Safe Rust emission with `#![forbid(unsafe_code)]`.

## Important limitations
- `try/catch` currently transports strings, not typed exception objects. No rethrow, filters, stack traces or full C# cleanup semantics.
- Only a narrow syntax subset; no inheritance, classes with fields, methods, interfaces, generics, async or full ownership inference.
- No actual C/C++ binding generator, cycle collector, hardware backend, installed VSIX or source-level debugger.
- No full MSBuild `.csxproj` build. Project template and build targets are scaffolding.
- Rust toolchain and Visual Studio SDK were unavailable in this environment; compilation not verified.

Never describe unimplemented features as working.

## Semantic LSP workspace index
A Python semantic subset index now implements multi-file class/member resolution, basic inheritance-aware completion, scoped local symbols, references, rename, and diagnostic checks. This is not a complete CSharpX semantic analyzer and is not yet backed by the Rust compiler AST. Eight Python tests pass.
