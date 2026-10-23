@echo off
setlocal
cd /d "%~dp0"
"%~dp0UnlimitedToolWorks.exe" team-compiler compile "%~dp0exports"
set "code=%errorlevel%"
pause
exit /b %code%
