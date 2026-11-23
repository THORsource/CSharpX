@echo off
setlocal
set "ROOT=%~dp0"
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" (
  echo [ERROR] Visual Studio 2022 not detected. Run Setup-SDK.bat.
  exit /b 1
)
set "VSINSTALL="
for /f "usebackq delims=" %%I in (`powershell.exe -NoProfile -Command "& $env:VSWHERE -latest -version '[17.0,18.0)' -requires Microsoft.Component.MSBuild -property installationPath"`) do set "VSINSTALL=%%I"
if not defined VSINSTALL (
  echo [ERROR] VS 2022 with MSBuild not detected. Run Setup-SDK.bat.
  exit /b 1
)
set "MSBUILD=%VSINSTALL%\MSBuild\Current\Bin\MSBuild.exe"
if not exist "%MSBUILD%" (
  echo [ERROR] MSBuild.exe missing at %MSBUILD%
  exit /b 1
)
"%MSBUILD%" "%ROOT%ide\visual-studio-2022\VSIX\CSharpX.VSIX.csproj" /restore /t:Build /p:Configuration=Release "/p:DevEnvDir=%VSINSTALL%\Common7\IDE\\" /verbosity:minimal
if errorlevel 1 exit /b %ERRORLEVEL%
echo [OK] Search ide\visual-studio-2022\VSIX\bin\Release for the .vsix.
exit /b 0
