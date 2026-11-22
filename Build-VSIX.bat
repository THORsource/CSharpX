@echo off
setlocal
set "ROOT=%~dp0"
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" (
  echo [ERROR] Visual Studio 2022 not detected. Run Setup-SDK.bat.
  exit /b 1
)
set "VSINSTALL="
for /f "usebackq delims=" %%I in (`"%VSWHERE%" -latest -version "[17.0,18.0)" -requires Microsoft.VisualStudio.Component.VSSDK -property installationPath`) do set "VSINSTALL=%%I"
if not defined VSINSTALL (
  echo [ERROR] VS 2022 extension development workload missing. Run Setup-SDK.bat.
  exit /b 1
)
set "MSBUILD=%VSINSTALL%\MSBuild\Current\Bin\MSBuild.exe"
if not exist "%MSBUILD%" (
  echo [ERROR] MSBuild.exe missing at %MSBUILD%
  exit /b 1
)
"%MSBUILD%" "%ROOT%ide\visual-studio-2022\VSIX\CSharpX.VSIX.csproj" /restore /t:Build /p:Configuration=Release /verbosity:minimal
if errorlevel 1 exit /b %ERRORLEVEL%
echo [OK] Search ide\visual-studio-2022\VSIX\bin\Release for the .vsix.
exit /b 0
