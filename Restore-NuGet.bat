@echo off
setlocal
cd /d "%~dp0"
if "%~1"=="" (
  echo Usage: Restore-NuGet.bat path\to\project.csxproj
  exit /b 2
)
where py >nul 2>&1
if not errorlevel 1 (
  py -3 "%~dp0tools\csx-dotnet\bridge.py" "%~1"
) else (
  python "%~dp0tools\csx-dotnet\bridge.py" "%~1"
)
exit /b %errorlevel%
