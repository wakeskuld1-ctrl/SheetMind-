@echo off
setlocal

rem 2026-04-22 CST: Added a single root launcher so customer delivery checks
rem can start from one obvious file while the implementation stays inside src.

set "ROOT_DIR=%~dp0"
set "PS_SCRIPT=%ROOT_DIR%src\delivery_check.ps1"

if not exist "%PS_SCRIPT%" (
  echo Missing src\delivery_check.ps1
  pause
  exit /b 1
)

powershell -ExecutionPolicy Bypass -File "%PS_SCRIPT%"
set "EXIT_CODE=%ERRORLEVEL%"

echo.
echo Delivery check finished with exit code %EXIT_CODE%.
echo Please send the newest zip or report folder under src\reports back.
pause
exit /b %EXIT_CODE%
