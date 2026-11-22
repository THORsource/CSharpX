# Tooling and diagnostics (DRAFT)

Stable command surface: `csxc check`, `build`, `test`, `emit-rust`, `api --format json`, `explain <code>`. Only `check`, `build`, `emit-rust` on a tiny subset are currently implemented.

Diagnostic families: CSX1xxx syntax/types, CSX2xxx inheritance, CSX3xxx ownership, CSX4xxx collection, CSX5xxx embedded/effects, CSX6xxx target, CSX7xxx backend, CSX9xxx toolchain.

Language server must share compiler frontend and expose completion, hover, definition, references, rename, formatting, semantic tokens, diagnostics, code actions and signature help. Debugger requires CSharpX-level symbol/source maps, stack/variable visualization and native debug adapters.
