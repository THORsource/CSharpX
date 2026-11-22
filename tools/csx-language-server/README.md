# CSharpX prototype language server

Start: `python server.py` (stdio, LSP Content-Length framing). No third-party Python dependencies.

Features: initialize, shutdown, text sync, completion, go to definition, references, hover, document symbols, rename, brace diagnostics. It is a lexical prototype, **not** a semantic compiler service.

Tests: `python -m unittest -v`.
