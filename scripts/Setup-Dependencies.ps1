param(
    [switch]$CheckOnly,
    [switch]$SkipVisualStudio,
    [switch]$SkipRust,
    [switch]$SkipPython
)

$ErrorActionPreference = 'Stop'

function Update-ToolPath {
    $persistent = [Environment]::GetEnvironmentVariable('Path', 'Machine'),
                  [Environment]::GetEnvironmentVariable('Path', 'User')
    $entries = @($env:Path) + $persistent + @(
        (Join-Path $env:USERPROFILE '.cargo\bin'),
        (Join-Path $env:ProgramFiles 'dotnet'),
        (Join-Path $env:LOCALAPPDATA 'Programs\Python\Python312')
    )
    $env:Path = (($entries -split ';' | Where-Object { $_ } | Select-Object -Unique) -join ';')
}

function Find-VisualStudio {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (!(Test-Path -LiteralPath $vswhere)) { return }
    & $vswhere -latest -version '[17.0,18.0)' -property installationPath | Select-Object -First 1
}

function Get-SetupState {
    Update-ToolPath
    $dotnet = Get-Command dotnet.exe -CommandType Application -ErrorAction SilentlyContinue
    $python = Get-Command python.exe -All -CommandType Application -ErrorAction SilentlyContinue |
        Where-Object { $_.Source -notlike '*\Microsoft\WindowsApps\*' } | Select-Object -First 1
    $vs = if (!$SkipVisualStudio) { Find-VisualStudio }
    $vc = if ($vs) { Join-Path $vs 'VC\Auxiliary\Build\vcvars64.bat' }
    $vsSdk = if ($vs) { Join-Path $vs 'Common7\IDE\CommonExtensions\Microsoft\Project\Microsoft.VisualStudio.ProjectSystem.dll' }
    $packs = Join-Path ${env:ProgramFiles(x86)} 'Reference Assemblies\Microsoft\Framework\.NETFramework\v4.7.2\mscorlib.dll'
    [pscustomobject]@{
        DotNet = [bool]($dotnet -and (& $dotnet.Source --list-sdks | Where-Object { $_ -match '^9\.0\.' }))
        Python = [bool]($SkipPython -or $python)
        Rust = [bool]($SkipRust -or ((Get-Command cargo.exe -ErrorAction SilentlyContinue) -and (Get-Command rustc.exe -ErrorAction SilentlyContinue)))
        VisualStudio = $vs
        VisualStudioReady = [bool]($SkipVisualStudio -or ($vs -and (Test-Path -LiteralPath $vc) -and
            (Test-Path -LiteralPath $vsSdk) -and (Test-Path -LiteralPath $packs)))
    }
}

function Install-Package([string]$Id) {
    $winget = Get-Command winget.exe -ErrorAction SilentlyContinue
    if (!$winget) { throw 'winget fehlt. Installiere Microsoft App Installer oder die angezeigten Voraussetzungen manuell.' }
    Write-Host "Installiere $Id ..."
    & $winget.Source install --id $Id --exact --source winget --accept-source-agreements --accept-package-agreements
    if ($LASTEXITCODE) { throw "Installation von $Id fehlgeschlagen (Code $LASTEXITCODE)." }
}

$state = Get-SetupState
foreach ($item in @(@('.NET-9-SDK',$state.DotNet), @('Python 3',$state.Python),
                    @('Rust und Cargo',$state.Rust), @('Visual Studio 2022 mit Buildtools',$state.VisualStudioReady))) {
    $status = if ($item[1]) { 'OK' } else { 'FEHLT' }
    Write-Host "  [$status] $($item[0])"
}
if ($state.DotNet -and $state.Python -and $state.Rust -and $state.VisualStudioReady) { exit 0 }
if ($CheckOnly) { exit 2 }
if (!$SkipVisualStudio -and !$state.VisualStudio) {
    Write-Host 'Visual Studio 2022 fehlt. Der Assistent kann die Community-Ausgabe installieren.'
}
if ((Read-Host 'Fehlende Voraussetzungen installieren? [j/N]') -notmatch '^(j|ja)$') { exit 2 }
if (!$state.DotNet) { Install-Package 'Microsoft.DotNet.SDK.9' }
if (!$state.Python) { Install-Package 'Python.Python.3.12' }
if (!$state.Rust) { Install-Package 'Rustlang.Rustup' }
if (!$SkipVisualStudio -and !$state.VisualStudio) {
    Install-Package 'Microsoft.VisualStudio.2022.Community'
    $state = Get-SetupState
}
if (!$state.VisualStudioReady) {
    if (!$state.VisualStudio) { throw 'Visual Studio 2022 wurde noch nicht gefunden. Starte den Assistenten nach der Installation erneut.' }
    $installer = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\setup.exe'
    Write-Host 'Visual Studio Installer ergaenzt C++-Desktopentwicklung und Visual-Studio-Erweiterungsentwicklung.'
    $arguments = @('modify','--installPath', ('"{0}"' -f $state.VisualStudio),
        '--add','Microsoft.VisualStudio.Workload.NativeDesktop',
        '--add','Microsoft.VisualStudio.Workload.VisualStudioExtension',
        '--add','Microsoft.Net.Component.4.7.2.TargetingPack',
        '--includeRecommended','--passive','--norestart')
    $process = Start-Process -FilePath $installer -ArgumentList $arguments -Wait -PassThru
    if ($process.ExitCode -eq 3010) {
        Write-Host 'Starte Windows neu und fuehre den Installationsassistenten erneut aus.'
        exit 2
    }
    if ($process.ExitCode) { throw "Visual Studio Installer meldet Code $($process.ExitCode)." }
}
$state = Get-SetupState
if (!$state.DotNet -or !$state.Python -or !$state.Rust -or !$state.VisualStudioReady) {
    throw 'Voraussetzungen noch nicht vollstaendig. Oeffne ein neues Terminal und starte den Assistenten erneut.'
}
exit 0
