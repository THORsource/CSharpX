@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>nul
if errorlevel 1 (
  echo [ERROR] cargo not found. Run Setup-SDK.bat and open a new terminal.
  exit /b 1
)
cargo build --workspace --release
if errorlevel 1 exit /b %ERRORLEVEL%
echo Compiler: %~dp0target\release\csxc.exe
exit /b 0
