param([Parameter(Mandatory = $true)][string]$Csxc)
$ErrorActionPreference = 'Stop'
$Csxc = (Resolve-Path -LiteralPath $Csxc).Path
Push-Location $PSScriptRoot
try {
    New-Item -ItemType Directory -Force build | Out-Null
    $env:DOTNET_CLI_HOME = Join-Path $PSScriptRoot 'build/dotnet-home'
    $env:DOTNET_GENERATE_ASPNET_CERTIFICATE = 'false'
    $env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
    [IO.File]::WriteAllLines((Join-Path $PSScriptRoot 'build/inputs.txt'), [string[]]@((Join-Path $PSScriptRoot '../Program.csx')), [Text.UTF8Encoding]::new($false))
    & $Csxc build-project build/inputs.txt build/generated.rs build/primes-csx.exe --optimize
    if ($LASTEXITCODE) { throw 'CSharpX build failed.' }
    rustc -C opt-level=3 primes.rs -o build/primes-rust.exe
    if ($LASTEXITCODE) { throw 'Rust build failed.' }
    dotnet build Primes.csproj -c Release -o build/dotnet --nologo
    if ($LASTEXITCODE) { throw 'C# build failed.' }

    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$vs) { throw 'Visual Studio with C++ build tools is required.' }
    $vcvars = Join-Path $vs 'VC/Auxiliary/Build/vcvars64.bat'
    @"
@echo off
call "$vcvars" >nul
if errorlevel 1 exit /b 1
cl /nologo /W4 /O2 /std:c11 /Fo:build\primes.obj /Fe:build\primes-c.exe primes.c
"@ | Set-Content -Encoding ASCII build/build-c.cmd
    cmd /d /c build\build-c.cmd
    if ($LASTEXITCODE) { throw 'C build failed.' }
    python benchmark.py
    if ($LASTEXITCODE) { throw 'Benchmark failed.' }
} finally {
    Pop-Location
}
