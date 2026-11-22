# C#-compatible using declarations in CSharpX

## Supported in this incremental language-server prototype

- `using Company.Devices;` imports classes from that namespace for completion and navigation.
- `using Sensor = Company.Devices.Sensor;` supports aliases for type navigation.
- `namespace Company.Devices { ... }` and `namespace Company.Devices;` associate class declarations with namespaces.
- Existing global-namespace classes continue to work.

## Planned compiler semantics (NOT implemented yet)

- `using static Namespace.Type;`, `global using`, nested namespaces, `extern alias`, generic and qualified name resolution, alias shadowing and diagnostics.
- Namespace visibility must be handled by the compiler parser/type checker, not just the editor.
- `using System;` and `using System.Collections.Generic;` require actual CSharpX standard-library definitions; they do not import .NET binaries.
- Native .NET assembly references would require a separately specified .NET bridge or a managed execution target. Arbitrary .NET assemblies cannot be directly linked to native Rust binaries.
- Namespace ambiguity must be a diagnostic rather than silently selecting an arbitrary class.

## Example

```csharp
using Company.Devices;
namespace Application;
class Program { void Run() { Sensor sensor; } }
```

This update affects the Python semantic language server and its VSIX copy, **not** the Rust compiler's parser. A VSIX build and a real VS2022 test have not been performed.
