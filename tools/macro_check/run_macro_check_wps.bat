@echo off
setlocal

rem 2026-04-22 CST: Added a WPS-forced entrypoint so mixed Office/WPS
rem machines can validate WPS without relying on default file association.

set "SCRIPT_DIR=%~dp0"
set "PS_SCRIPT=%SCRIPT_DIR%macro_check.ps1"

if not exist "%PS_SCRIPT%" (
  echo Missing macro_check.ps1 in %SCRIPT_DIR%
  exit /b 1
)

powershell -ExecutionPolicy Bypass -File "%PS_SCRIPT%" -PreferredHost wps
set "EXIT_CODE=%ERRORLEVEL%"

if not "%EXIT_CODE%"=="0" (
  echo WPS macro check failed with exit code %EXIT_CODE%.
)

exit /b %EXIT_CODE%
