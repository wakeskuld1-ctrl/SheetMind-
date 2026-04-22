@echo off
setlocal

rem 2026-04-22 CST: Added a single double-click entrypoint so customer-side
rem macro startup checks do not require manual PowerShell commands.

set "SCRIPT_DIR=%~dp0"
set "PS_SCRIPT=%SCRIPT_DIR%macro_check.ps1"

if not exist "%PS_SCRIPT%" (
  echo Missing macro_check.ps1 in %SCRIPT_DIR%
  exit /b 1
)

where powershell >nul 2>nul
if errorlevel 1 (
  echo PowerShell was not found on this machine.
  exit /b 90
)

powershell -NoProfile -Command "$v=$Host.Version; if (($v.Major -lt 5) -or (($v.Major -eq 5) -and ($v.Minor -lt 1))) { exit 51 }"
if "%ERRORLEVEL%"=="51" (
  echo Windows PowerShell 5.1 or newer is required.
  exit /b 51
)

powershell -ExecutionPolicy Bypass -File "%PS_SCRIPT%"
set "EXIT_CODE=%ERRORLEVEL%"

if not "%EXIT_CODE%"=="0" (
  echo Macro check failed with exit code %EXIT_CODE%.
)

exit /b %EXIT_CODE%
