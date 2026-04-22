@echo off
setlocal

rem 2026-04-22 CST: Added a single root launcher so customer environment
rem prechecks can start from one obvious file while logic stays inside src.

set "ROOT_DIR=%~dp0"
set "PS_SCRIPT=%ROOT_DIR%src\precheck.ps1"

if not exist "%PS_SCRIPT%" (
  echo Missing src\precheck.ps1
  pause
  exit /b 1
)

powershell -ExecutionPolicy Bypass -File "%PS_SCRIPT%"
set "EXIT_CODE=%ERRORLEVEL%"

if not "%EXIT_CODE%"=="0" (
  echo.
  echo Precheck failed with exit code %EXIT_CODE%.
  echo Please contact us and send the newest folder under src\reports back.
  pause
)

exit /b %EXIT_CODE%
