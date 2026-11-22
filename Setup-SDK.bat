@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\Setup-Dependencies.ps1" %*
exit /b %ERRORLEVEL%
