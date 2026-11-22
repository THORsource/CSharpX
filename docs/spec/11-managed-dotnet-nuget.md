# Managed .NET DLL and NuGet integration contract (design + initial tooling)

## Goal
CSharpX shall accept `using` namespaces from referenced .NET assemblies and NuGet packages, with C#-style public API names. The existing Rust-native compiler **cannot currently execute .NET IL**. NuGet packages are not equivalent to native C/C++ DLLs.

## Proposed project format

```xml
<ItemGroup>
  <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
  <Reference Include="LegacyLib"><HintPath>lib\LegacyLib.dll</HintPath></Reference>
</ItemGroup>
```

## Current implementation
`tools/csx-dotnet/bridge.py` parses PackageReference and direct Reference/HintPath, creates a temporary .NET SDK restore project, invokes `dotnet restore`, reads NuGet `project.assets.json` and writes `obj/csharpx-dotnet-references.json` with compile/runtime assembly asset locations. `Restore-NuGet.bat` invokes the tool. This is an **assembly inventory**, not executable managed interop, and does not add symbols to the current compiler.

## Required next stages
1. Metadata reader for ECMA-335 assemblies (types, members, generics, visibility, attributes, nullable metadata).
2. Import symbol tables into the same semantic engine used by `csxc` and the VS language server.
3. A .NET host mode for desktop: initialize CoreCLR via supported hosting APIs, load managed assemblies in an appropriate AssemblyLoadContext, marshal arguments, manage GC handles and exception translation. Alternatively implement an explicit IPC bridge for process isolation.
4. Define an ABI-neutral interop contract for strings, arrays, delegates, async, exceptions, ownership and callbacks. Ensure CLR GC roots remain alive across native calls.
5. Distinguish NuGet package assets: managed assemblies, native runtime assets, analyzers, build targets and source generators; not every package can be supported automatically.
6. Keep bare-metal profiles free of CLR dependencies; reject unsupported managed references at compile time.
7. Audit trust boundaries: native-to-CLR hosting may use audited unsafe runtime code, but ordinary generated CSharpX code must not introduce Rust `unsafe` without explicit CSharpX `unsafe`.

## Compatibility limits
No claim of universal .NET DLL compatibility is made. Mixed-mode assemblies, Windows-specific APIs, dynamic code generation, reflection emit, COM, C++/CLI and framework-dependent packages require separate support or restrictions. NuGet restore requires a local .NET SDK and package source access.
