@echo off

echo Starting Installation

REM C:\Users\x\AppData\Local\software.Browsers\

REM TODO: would be even more correct to take from registry
set LocalProgramsDir=%LocalAppData%\Programs
set ProgramDir=%LocalProgramsDir%\software.Browsers

mkdir "%ProgramDir%"
copy "aarch64\browsers.exe" "%ProgramDir%\browsers.exe"

mkdir "%ProgramDir%\resources\icons\512x512"
copy "icons\512x512\software.Browsers.png" "%ProgramDir%\resources\icons\512x512\software.Browsers.png"

mkdir "%ProgramDir%\resources\i18n\en-US"
copy "i18n\en-US\builtin.ftl" "%ProgramDir%\resources\i18n\en-US\builtin.ftl"

REM C:\Users\x\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Browsers\Browsers.lnk
mkdir "%AppData%\Microsoft\Windows\Start Menu\Programs\Browsers"
copy "startmenu\Browsers.lnk" "%AppData%\Microsoft\Windows\Start Menu\Programs\Browsers\Browsers.lnk"

echo Installation finished!
