# Language Server — planned component

Transport: LSP 3.17 over stdio. Required methods: initialize, initialized, textDocument/didOpen, didChange, didClose, completion, hover, definition, references, rename, signatureHelp, documentSymbol, semanticTokens/full, codeAction, formatting, publishDiagnostics.

Reuse the compiler AST and symbol index; never implement independent language semantics in the editor. UTF-16 positions, cancellation, incremental documents and cross-file project graphs are mandatory for production.

**Not implemented in this package.**
