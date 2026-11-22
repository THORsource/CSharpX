@echo off
setlocal
call "%~dp0Build-Compiler.bat"
if errorlevel 1 exit /b %ERRORLEVEL%
call "%~dp0Build-VSIX.bat"
if errorlevel 1 exit /b %ERRORLEVEL%
echo [OK] Compiler and VSIX build completed.
exit /b 0
