param([switch]$SkipVisualStudio, [switch]$SkipRust, [switch]$SkipPython)
$ErrorActionPreference = 'Stop'
function Info($s) { Write-Host "[CSharpX] $s" -ForegroundColor Cyan }
function Require-Winget { if (-not (Get-Command winget -ErrorAction SilentlyContinue)) { throw 'winget is missing. Install Microsoft App Installer from Microsoft Store or install dependencies manually.' } }
function Install-Package($id) {
  Require-Winget
  Info "Installing $id (if needed)..."
  & winget install --id $id --exact --source winget --accept-source-agreements --accept-package-agreements --silent
  if ($LASTEXITCODE -ne 0) { throw "winget install failed for $id (exit $LASTEXITCODE). Check installer output." }
}
if (-not (Get-Command dotnet -ErrorAction SilentlyContinue)) { Install-Package 'Microsoft.DotNet.SDK.8' }
if (-not $SkipPython -and -not (Get-Command py -ErrorAction SilentlyContinue) -and -not (Get-Command python -ErrorAction SilentlyContinue)) { Install-Package 'Python.Python.3.12' }
if (-not $SkipRust -and -not (Get-Command cargo -ErrorAction SilentlyContinue)) { Install-Package 'Rustlang.Rustup' }
if (-not $SkipVisualStudio) {
  $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
  if (-not (Test-Path $vswhere)) { throw 'Visual Studio 2022 not found. Install VS 2022 Community/Professional/Enterprise first, then rerun. VSIX builds require Visual Studio SDK.' }
  $vs = & $vswhere -latest -version '[17.0,18.0)' -products 'Microsoft.VisualStudio.Product.Community' 'Microsoft.VisualStudio.Product.Professional' 'Microsoft.VisualStudio.Product.Enterprise' -property installationPath
  if (-not $vs) { throw 'No Visual Studio 2022 IDE found. Install it first.' }
  $installer = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\setup.exe'
  if (-not (Test-Path $installer)) { throw 'Visual Studio Installer setup.exe not found.' }
  $components = & $vswhere -latest -version '[17.0,18.0)' -requires Microsoft.VisualStudio.Component.VSSDK -property installationPath
  if (-not $components) {
    Info 'Adding Visual Studio extension development workload. Administrator approval may be required.'
    $proc = Start-Process -FilePath $installer -ArgumentList @('modify','--installPath',"`"$vs`"",'--add','Microsoft.VisualStudio.Workload.VisualStudioExtension','--includeRecommended','--passive','--norestart') -Wait -PassThru
    if ($proc.ExitCode -notin @(0,3010)) { throw "Visual Studio workload installer failed (exit $($proc.ExitCode))." }
    if ($proc.ExitCode -eq 3010) { Write-Warning 'Restart Windows before building.' }
  } else { Info 'Visual Studio extension development SDK already installed.' }
}
Info 'Setup finished. Open a NEW terminal to refresh PATH, then run Build-All.bat.'
