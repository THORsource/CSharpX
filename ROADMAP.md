# CSharpX implementation roadmap

## Delivered in expanded prototype
- Single-file tokenizer, parser, expression lowering, basic static types, string exceptions, safe Rust emission.
- Extended semantic/FFI/exception specifications, target descriptors and VSIX project scaffold.

## Required for a real compiler
1. Source spans and structured diagnostics; full lexer/grammar and semantic model.
2. Multi-file namespaces, classes, inheritance, interfaces, virtual dispatch and generics.
3. Ownership/escape/effect analysis; managed object model and cycle collector.
4. Complete C# exception semantics, async, debug/source mappings.
5. Clang-based C/C++ bindings, ABI validation, native build/link orchestration.
6. Real target backends and embedded HAL/startup/linker/debug integration.
7. LSP server, VSIX packaging, CPS/MSBuild, debugging and project templates.
8. Conformance suites, fuzzing, security audits, CI and documentation generation.

Do not claim these roadmap items are implemented.

## Increment: inheritance and ownership prototype
Implemented: single inheritance for integer-literal-returning methods, virtual override dispatch, simple upcasts, static local aliases, Box/Rc classification and an ownership report. Next: fields and constructors, flow-sensitive SSA alias/escape analysis, safe object graph representation, cycle collection, integration with exceptions and full parser.
