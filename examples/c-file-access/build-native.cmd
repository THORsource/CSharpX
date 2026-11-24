@echo off
setlocal DisableDelayedExpansion
set "output=%~1"
set "source_list=%~2"
if not defined output exit /b 1
if not exist "%source_list%" exit /b 1
set "vswhere=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
for /f "usebackq delims=" %%i in (`"%vswhere%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "vcvars=%%i\VC\Auxiliary\Build\vcvars64.bat"
if not defined vcvars (
  echo error CSX1704: Install the Visual Studio x64 C build tools.
  exit /b 1
)
call "%vcvars%" >nul
if errorlevel 1 exit /b 1
set "objects=%output%\objects.rsp"
type nul > "%objects%"
set /a index=0
for /f "usebackq delims=" %%s in ("%source_list%") do (
  set "source=%%s"
  call :compile_source
  if errorlevel 1 exit /b 1
)
if %index% equ 0 exit /b 1
lib /nologo /OUT:"%output%\csx_c_file_native.lib" @"%objects%"
exit /b %errorlevel%

:compile_source
set /a index+=1
set "object=%output%\unit-%index%.obj"
cl /nologo /W4 /WX /std:c11 /MD /DCSX_C_FILE_LIBRARY /c /Fo"%object%" "%source%"
if errorlevel 1 exit /b 1
>> "%objects%" echo "%object%"
exit /b %errorlevel%
