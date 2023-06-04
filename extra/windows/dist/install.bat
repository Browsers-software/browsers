@echo off

echo Starting Installation
setlocal enabledelayedexpansion

REM .bat location with trailing \
set THIS_DIR=%~dp0

REM Sets ARCH to ARM64 or AMD64
for /f "tokens=3" %%a in ('reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v PROCESSOR_ARCHITECTURE ^| findstr /ri "REG_SZ"') do set ARCH_WIN=%%a

REM echo %ARCH_WIN%

if "%ARCH_WIN%" == "ARM64" (set ARCH=aarch64)
if "%ARCH_WIN%" == "AMD64" (set ARCH=x86_64)

REM echo %ARCH%

set SRC_BINARY_PATH=%THIS_DIR%%ARCH%\browsers.exe

if exist "%windir%\system32\config\systemprofile\*" (
  set is_admin=true
) else (
  set is_admin=false
)

if "%~1"=="--system" (
  if %is_admin% == false (
    echo You must run this installer with Administrator privileges when using --system flag
    echo Please run as administrator (no --system required then^)
    echo.

    exit /b 1
  )

  set is_local_install=false
) else (
  set is_local_install=true
)

if %is_admin% == true (
  echo Because you are running this as an administrator we are going to install it to the whole system
  echo.
  set is_local_install=false
)

REM C:\Users\x\AppData\Local\software.Browsers\

if %is_local_install% == true (
    REM TODO: would be even more correct to take from registry
    set LocalProgramsDir=%LocalAppData%\Programs
    set ProgramDir=!LocalProgramsDir!\software.Browsers
) else (
    set ProgramDir=%ProgramFiles%\software.Browsers
)

if not exist "%ProgramDir%\" (
  mkdir "%ProgramDir%" || exit /b
)

copy "%SRC_BINARY_PATH%" "%ProgramDir%\browsers.exe" 1>nul

if not exist "%ProgramDir%\resources\icons\512x512\" (
  mkdir "%ProgramDir%\resources\icons\512x512" || exit /b
)

copy "%THIS_DIR%icons\512x512\software.Browsers.png" "%ProgramDir%\resources\icons\512x512\software.Browsers.png" 1>nul

if not exist "%ProgramDir%\resources\i18n\en-US\" (
  mkdir "%ProgramDir%\resources\i18n\en-US" || exit /b
)

copy "%THIS_DIR%i18n\en-US\builtin.ftl" "%ProgramDir%\resources\i18n\en-US\builtin.ftl" 1>nul

REM C:\Users\x\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Browsers\Browsers.lnk

if not exist "%AppData%\Microsoft\Windows\Start Menu\Programs\Browsers\" (
  mkdir "%AppData%\Microsoft\Windows\Start Menu\Programs\Browsers" || exit /b
)

REM TODO: use different shortcut if system install
REM TODO: add shortcut for all users start menu dir if system install
copy "%THIS_DIR%startmenu\Browsers.lnk" "%AppData%\Microsoft\Windows\Start Menu\Programs\Browsers\Browsers.lnk" 1>nul

REG ADD "HKCU\Software\Classes\software.Browsers" /ve /d "Browsers HTML Document" /f 1>nul
REG ADD "HKCU\Software\Classes\software.Browsers" /v AppUserModelId /t REG_SZ /d "software.Browsers" /f 1>nul

REG ADD "HKCU\Software\Classes\software.Browsers\Application" /v AppUserModelId /t REG_SZ /d "software.Browsers" /f 1>nul
REG ADD "HKCU\Software\Classes\software.Browsers\Application" /v ApplicationIcon /t REG_SZ /d "%LocalAppData%\Programs\software.Browsers\browsers.exe,0" /f 1>nul
REG ADD "HKCU\Software\Classes\software.Browsers\Application" /v ApplicationName /t REG_SZ /d "Browsers" /f 1>nul
REG ADD "HKCU\Software\Classes\software.Browsers\Application" /v ApplicationDescription /t REG_SZ /d "Open the right browser at the right time" /f 1>nul
REG ADD "HKCU\Software\Classes\software.Browsers\Application" /v ApplicationCompany /t REG_SZ /d "Browsers.software team" /f 1>nul

REG ADD "HKCU\Software\Classes\software.Browsers\DefaultIcon" /ve /d "%LocalAppData%\Programs\software.Browsers\browsers.exe,0" /f 1>nul

REG ADD "HKCU\Software\Classes\software.Browsers\shell\open\command" /ve /d "\"%LocalAppData%\Programs\software.Browsers\browsers.exe\" \"%%1\"" /f 1>nul

REG ADD "HKCU\Software\Clients\StartMenuInternet\software.Browsers" /ve /d "Browsers" /f 1>nul

REG ADD "HKCU\Software\Clients\StartMenuInternet\software.Browsers\Capabilities" /v ApplicationIcon /t REG_SZ /d "%LocalAppData%\Programs\software.Browsers\browsers.exe,0" /f 1>nul

REG ADD "HKCU\Software\Clients\StartMenuInternet\software.Browsers\Capabilities" /v ApplicationName /t REG_SZ /d "Browsers" /f 1>nul
REG ADD "HKCU\Software\Clients\StartMenuInternet\software.Browsers\Capabilities" /v ApplicationDescription /t REG_SZ /d "Open the right browser at the right time" /f 1>nul

REG ADD "HKCU\Software\Clients\StartMenuInternet\software.Browsers\Capabilities\URLAssociations" /v http /t REG_SZ /d "software.Browsers" /f 1>nul
REG ADD "HKCU\Software\Clients\StartMenuInternet\software.Browsers\Capabilities\URLAssociations" /v https /t REG_SZ /d "software.Browsers" /f 1>nul

REG ADD "HKCU\Software\Clients\StartMenuInternet\software.Browsers\DefaultIcon" /ve /d "%LocalAppData%\Programs\software.Browsers\browsers.exe,0" /f 1>nul

REG ADD "HKCU\Software\Clients\StartMenuInternet\software.Browsers\shell\open\command" /ve /d "\"%LocalAppData%\Programs\software.Browsers\browsers.exe\"" /f 1>nul

REG ADD "HKCU\Software\RegisteredApplications" /v software.Browsers /t REG_SZ /d "Software\Clients\StartMenuInternet\software.Browsers\Capabilities" /f 1>nul

REG ADD "HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths\browsers.exe" /ve /d "%LocalAppData%\Programs\software.Browsers\browsers.exe" /f 1>nul
REG ADD "HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths\browsers.exe" /v Path /t REG_SZ /d "%LocalAppData%\Programs\software.Browsers" /f 1>nul

REG ADD "HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths\browsers.exe\SupportedProtocols" /v http /t REG_SZ /d "" /f 1>nul
REG ADD "HKCU\Software\Microsoft\Windows\CurrentVersion\App Paths\browsers.exe\SupportedProtocols" /v https /t REG_SZ /d "" /f 1>nul

powershell -ExecutionPolicy Bypass -File %THIS_DIR%announce_default.ps1

echo Browsers has been installed. Enjoy!
echo Please report any issues at https://github.com/Browsers-software/browsers/issues

echo You can now press Enter to exit this installer.
set /p input=
