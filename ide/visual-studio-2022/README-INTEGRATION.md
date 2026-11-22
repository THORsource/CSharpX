# CSharpX Visual Studio 2022 — integration prototype

## Delivered
- `.csx` content type and lexical highlighting from previous VSIX scaffold.
- Python standard-library LSP process with completion, go-to-definition, references, hover, document symbols, rename, brace diagnostics.
- C# `ILanguageClient` adapter that launches `server.py` via stdio.
- MSBuild `CSharpX.targets` with `CSharpXCheck` and `CSharpXNativeBuild` targets.
- Unit tests for basic language-server parsing.

## Build prerequisites
Windows, Visual Studio 2022, Visual Studio extension development workload, Python 3 available as `python`, and Rust toolchain (`cargo`/`rustc`) for compiler. The VSIX must be built on Windows with VS SDK dependencies. Exact SDK compatibility is not validated in this environment.

## Build and install
1. Open `VSIX/CSharpX.VSIX.csproj` in Visual Studio 2022.
2. Restore NuGet dependencies; resolve any SDK-specific interface version differences.
3. Build Release, then install resulting VSIX in an experimental Visual Studio instance.
4. If needed set `CSHARPX_LSP_PYTHON` to full path of python.exe and `CSHARPX_LSP_PATH` to `server.py`.
5. Open a `.csx` file and test completion / F12 / Find References.

## MSBuild compiler integration
Import `Build/CSharpX.targets` into an MSBuild project and set `CSharpXCompiler` to the compiled `csxc.exe` path and `CSharpXInput` to a `.csx` file. `CSharpXCheck` runs before Build. `CSharpXNativeBuild` is an explicit target to compile with rustc. The current compiler accepts one `.csx` file only; full multi-file `.csxproj` compilation is not implemented.

Example:
```xml
<Project DefaultTargets="CSharpXNativeBuild" xmlns="http://schemas.microsoft.com/developer/msbuild/2003">
  <PropertyGroup>
    <CSharpXCompiler>C:\\tools\\csxc.exe</CSharpXCompiler>
    <CSharpXInput>$(MSBuildProjectDirectory)\\Program.csx</CSharpXInput>
  </PropertyGroup>
  <Import Project="Build\\CSharpX.targets" />
</Project>
```

## Known limitations
- Not a verified, installable VSIX binary; Visual Studio SDK unavailable here.
- LSP parser is heuristic: no project-wide symbol resolution, inheritance-aware completion, type inference, robust lexer, or compiler-generated diagnostics yet.
- LSP `textDocumentSync` uses full-document changes. VSIX language client compatibility must be validated against actual installed VS SDK.
- `CSharpXCheck` only supports the compiler's existing language subset and single-file input.
- Debugger, true CPS project system, templates with full build integration and cross-target tooling remain pending.

## Semantic workspace index (November 2026)
- `semantic.py` provides an incremental per-file declaration index and workspace-wide lookup for `.csx` files.
- On LSP initialization, `rootUri` and `workspaceFolders` are scanned; open document buffers replace indexed disk content.
- Resolution supports class names, inherited fields and methods, local variables, parameters, and `this` / `base` receiver member access in a restricted grammar.
- Definition, references, rename, completion, hover, document symbols, and signature help use this index.
- Diagnostics include unknown base classes, inheritance cycles, invalid overrides, and unmatched braces.
- Run `cd tools/csx-language-server && python -m unittest discover -v`.

**Conformance limit:** This is NOT a complete semantic engine. It does not fully parse namespaces, generic types, overload resolution, access modifiers, C# expressions, partial classes, interfaces, implicit conversions, or method type inference. It does not yet share the Rust compiler's semantic model. Treat unsupported constructs as unresolved rather than proof of validity. The existing VSIX and MSBuild integration have not been built in Visual Studio 2022 in this environment.
