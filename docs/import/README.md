# CSharpX — expanded experimental compiler and design package

CSharpX is a proposed C#-like native/embedded language with Rust as a hidden implementation backend. **This repository is an experimental single-file compiler subset, not a finished C# implementation.**

## Commands (Rust toolchain required)
```sh
cargo test --workspace
cargo run -p csxc -- check examples/expressions/Program.csx
cargo run -p csxc -- emit-rust examples/expressions/Program.csx
cargo run -p csxc -- build examples/expressions/Program.csx
```
The `build` command writes an executable to `csx-out/` and requires `rustc`.

## Current syntax
`public static void Main()`, optional class wrapper, integer and string literals, local variables, assignment, integer arithmetic, `Console.WriteLine`, string-valued `throw`, and basic `try/catch/finally`.

## Not implemented
Full C# classes/inheritance, managed memory/cycle collection, real C/C++ interop, embedded codegen, multi-file project build, production VSIX, debugger, LSP. These are documented as **design specifications**.

See [implementation status](docs/IMPLEMENTATION-STATUS.md), [roadmap](ROADMAP.md), [grammar](docs/spec/07-grammar-ebnf.md) and [AI entrypoint](docs/ai/llms.txt).

## New experimental inheritance and ownership subset
See [implementation details](docs/spec/10-inheritance-ownership-implementation.md) and `examples/inheritance/Program.csx`. The compiler now supports a restricted single-inheritance/virtual-method subset with a conservative Box-vs-Rc alias analysis. Run `csxc ownership-report examples/inheritance/Program.csx` to inspect its decisions. This does **not** implement full C# inheritance or cycle-collecting memory management.

## Managed DLL and NuGet groundwork
Run `Restore-NuGet.bat examples\nuget\NuGetDemo.csxproj` on Windows with .NET SDK and Python installed. This restores packages and inventories managed assemblies; execution of managed DLLs is **not yet implemented**. See `docs/spec/11-managed-dotnet-nuget.md`.
