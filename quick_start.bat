@echo off
echo Starting ThingsBoard Data Push Tool...
echo.

REM Run with default settings
target\release\push-message-thingsboard.exe

echo.
echo Press any key to exit...
pause >nul
