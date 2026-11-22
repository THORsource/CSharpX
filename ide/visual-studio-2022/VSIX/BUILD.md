# Build CSharpX VSIX (Visual Studio 2022)

## Requirements
- Windows with Visual Studio 2022 and **Visual Studio extension development** workload.
- .NET Framework 4.7.2 targeting pack; NuGet restore access.

## Build
Open `CSharpX.VSIX.csproj` in Visual Studio 2022 and build Release, or use the Developer PowerShell:

```powershell
msbuild .\CSharpX.VSIX.csproj /restore /p:Configuration=Release
```

Locate the generated `.vsix` under `bin\Release` and install by double-clicking. Restart Visual Studio.

## Implemented
- `.csx` file extension and editor content type registration
- Keyword, comment, string and number classification (regex-based prototype)
- Minimal VS package and VSIX metadata
- Project template that creates CSharpX source alongside a placeholder C# host project

## Not yet implemented
- LSP/IntelliSense, semantic navigation, real-time diagnostics, compiler build integration, native `.csxproj` project factory, debugger, code fixes, C/C++ binding UI, embedded flashing.
- The template's `.csxproj` is **not** a Visual Studio-recognized project type yet. The host `.csproj` only exposes source files; it does not compile CSharpX.

This VSIX is **source only and untested** in the current Linux environment. No installable compiled VSIX is supplied. VS SDK package versions and manifest assets may require adjustments in the target VS installation.
