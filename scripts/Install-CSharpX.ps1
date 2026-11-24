param([switch]$CheckOnly, [switch]$BuildOnly)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$setup = Join-Path $PSScriptRoot 'Setup-Dependencies.ps1'
$vsix = Join-Path $root 'ide\visual-studio-2022\VSIX\bin\Release\net472\CSharpX.VisualStudio.vsix'
Push-Location -LiteralPath $root
try {
    Write-Host 'CSharpX-Installation fuer Visual Studio 2022'
    Write-Host "`n[1/3] Voraussetzungen pruefen"
    $arguments = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $setup)
    if ($CheckOnly -or $BuildOnly) { $arguments += '-CheckOnly' }
    & powershell.exe @arguments
    if ($LASTEXITCODE -or $CheckOnly) { exit $LASTEXITCODE }
    # A child installer cannot update the parent process PATH.
    $entries = @($env:Path, [Environment]::GetEnvironmentVariable('Path', 'Machine'),
        [Environment]::GetEnvironmentVariable('Path', 'User'),
        (Join-Path $env:USERPROFILE '.cargo\bin'), (Join-Path $env:ProgramFiles 'dotnet'),
        (Join-Path $env:LOCALAPPDATA 'Programs\Python\Python312'))
    $env:Path = (($entries -split ';' | Where-Object { $_ } | Select-Object -Unique) -join ';')

    Write-Host "`n[2/3] Compiler und Visual-Studio-Erweiterung bauen"
    $buildDir = Join-Path $root 'target'
    New-Item -ItemType Directory -Force -Path $buildDir | Out-Null
    $log = Join-Path $buildDir 'install-build.log'
    $errors = Join-Path $buildDir 'install-build-errors.log'
    Write-Host "Bitte warten. Details: $log"
    $build = Start-Process -FilePath $env:ComSpec -ArgumentList '/d /c Build-All.bat' -WorkingDirectory $root -RedirectStandardOutput $log -RedirectStandardError $errors -WindowStyle Hidden -Wait -PassThru
    if ($build.ExitCode) { throw "Build fehlgeschlagen (Code $($build.ExitCode)). Protokolle: $log und $errors." }
    if (!(Test-Path -LiteralPath $vsix -PathType Leaf)) { throw "Die Erweiterung fehlt unter $vsix." }
    if ($BuildOnly) {
        Write-Host "Build abgeschlossen. Paket: $vsix"
        exit 0
    }

    Write-Host "`n[3/3] Visual-Studio-Erweiterung installieren"
    if ((Read-Host 'Erweiterung jetzt installieren? [j/N]') -notmatch '^(j|ja)$') {
        Write-Host "Build abgeschlossen. Das Installationspaket liegt unter $vsix."
        exit 0
    }
    while (Get-Process devenv -ErrorAction SilentlyContinue) {
        if ((Read-Host 'Bitte Visual Studio schliessen. [Enter] erneut pruefen, [A] abbrechen') -match '^a$') {
            exit 2
        }
    }
    & (Join-Path $root 'Install-VSIX.bat')
    if ($LASTEXITCODE) { throw "VSIX-Installation abgebrochen oder fehlgeschlagen (Code $LASTEXITCODE)." }
    Write-Host 'Installation abgeschlossen. Oeffne examples\hello\Hello.csxproj in Visual Studio 2022.'
} catch {
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
} finally {
    Pop-Location
}
