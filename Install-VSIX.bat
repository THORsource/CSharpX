@echo off
setlocal EnableDelayedExpansion
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" (echo [ERROR] Visual Studio Installer missing.& exit /b 1)
set "VSINSTALL="
for /f "usebackq delims=" %%I in (`"%VSWHERE%" -latest -version "[17.0,18.0)" -property installationPath`) do set "VSINSTALL=%%I"
if not defined VSINSTALL (echo [ERROR] VS2022 missing.& exit /b 1)
set "VSIXINSTALLER=%VSINSTALL%\Common7\IDE\VSIXInstaller.exe"
if not exist "%VSIXINSTALLER%" (echo [ERROR] VSIXInstaller missing.& exit /b 1)
set "VSIXFILE="
for /r "%~dp0ide\visual-studio-2022\VSIX\bin\Release" %%F in (*.vsix) do set "VSIXFILE=%%~fF"
if not defined VSIXFILE (echo [ERROR] No compiled VSIX found. Run Build-VSIX.bat first.& exit /b 1)
"%VSIXINSTALLER%" "!VSIXFILE!"
exit /b %ERRORLEVEL%
