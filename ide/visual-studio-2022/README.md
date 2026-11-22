# Visual Studio 2022 integration plan

## Current status

MSBuild target sketch is provided. No working VSIX, CPS project factory, LSP client, source-level debugger or IntelliSense implementation is included.

## Required components

1. VSIX project registration, `.csx` editor content type, syntax classification.
2. LSP client connecting to shared compiler-backed `csx-language-server`.
3. CPS-based `.csxproj` integration and MSBuild property pages.
4. Build/clean/rebuild, error list mapping and project templates.
5. CSharpX source maps and native debug adapter integration.
6. Ownership explorer, memory analyzer and optional generated-Rust viewer.

Use the Visual Studio 2022 SDK, Microsoft.VisualStudio.LanguageServer.Client and appropriate CPS APIs. APIs and extension model compatibility must be checked against the installed VS2022 SDK version.
