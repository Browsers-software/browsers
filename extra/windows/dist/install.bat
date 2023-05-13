@echo off

echo Starting Installation

REM Sets ARCH to ARM64 or AMD64
for /f "tokens=3" %%a in ('reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v PROCESSOR_ARCHITECTURE ^| findstr /ri "REG_SZ"') do set ARCH_WIN=%%a

REM echo %ARCH_WIN%

if "%ARCH_WIN%" == "ARM64" (set ARCH=aarch64)
if "%ARCH_WIN%" == "AMD64" (set ARCH=x86_64)

REM echo %ARCH%

set SRC_BINARY_PATH=%ARCH%\browsers.exe

REM C:\Users\x\AppData\Local\software.Browsers\

REM TODO: would be even more correct to take from registry
set LocalProgramsDir=%LocalAppData%\Programs
set ProgramDir=%LocalProgramsDir%\software.Browsers

if not exist "%ProgramDir%\" (
  mkdir "%ProgramDir%"
)

copy "%SRC_BINARY_PATH%" "%ProgramDir%\browsers.exe" 1>nul

if not exist "%ProgramDir%\resources\icons\512x512\" (
  mkdir "%ProgramDir%\resources\icons\512x512"
)

copy "icons\512x512\software.Browsers.png" "%ProgramDir%\resources\icons\512x512\software.Browsers.png" 1>nul

if not exist "%ProgramDir%\resources\i18n\en-US\" (
  mkdir "%ProgramDir%\resources\i18n\en-US"
)

copy "i18n\en-US\builtin.ftl" "%ProgramDir%\resources\i18n\en-US\builtin.ftl" 1>nul

REM C:\Users\x\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Browsers\Browsers.lnk

if not exist "%AppData%\Microsoft\Windows\Start Menu\Programs\Browsers\" (
  mkdir "%AppData%\Microsoft\Windows\Start Menu\Programs\Browsers"
)
copy "startmenu\Browsers.lnk" "%AppData%\Microsoft\Windows\Start Menu\Programs\Browsers\Browsers.lnk" 1>nul

echo Browsers has been installed. Enjoy!
echo Please report any issues at https://github.com/Browsers-software/browsers/issues

echo You can now press Enter to exit this installer.
set /p input=
