# Windows / Visual Studio 2022 build

Run from an **administrator terminal** when installing Visual Studio workloads:

1. `Setup-SDK.bat` – installs missing .NET 8 SDK, Python 3.12 and Rustup using winget; checks VS 2022 and adds the Visual Studio extension development workload if absent.
2. Close and reopen the terminal (and restart Windows if requested).
3. `Build-All.bat` – builds the Rust compiler and VSIX in Release mode.
4. `Install-VSIX.bat` – installs the built extension into Visual Studio 2022.

Individual steps: `Build-Compiler.bat`, `Build-VSIX.bat`.

## Caveats
- `Setup-SDK.bat` does **not** silently install the Visual Studio IDE; it requires an existing VS 2022 installation and adds its missing extension-development workload. Install VS 2022 first if necessary.
- The installer may require administrator privileges, network access and a reboot. `winget` may return a failure if an installer is blocked by enterprise policy.
- The .NET 8 SDK is **not** the same as the .NET Framework 4.7.2 targeting pack used by the VSIX project. The Visual Studio workload and NuGet restore must supply its build prerequisites. If MSBuild reports missing .NET Framework reference assemblies, add the 4.7.2 targeting pack via Visual Studio Installer.
- The language server runs Python and requires `python` on PATH (or `CSHARPX_LSP_PYTHON`).
- VSIX is a prototype and has not been built or tested on Windows in this environment. Source/API compatibility issues may remain.
- Native CSharpX `.csxproj` projects are not yet fully supported by the VS project system.
- These scripts do not install the CSharpX compiler globally. The compiler executable remains under `target\release`.
