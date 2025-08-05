@echo off
echo.
echo ==========================================
echo    ThingsBoard Data Push Tool
echo ==========================================
echo.

REM Check if exe file exists
if not exist "target\release\push-message-thingsboard.exe" (
    echo Error: Executable file not found!
    echo Please run build.bat first to build the project
    echo.
    pause
    exit /b 1
)

REM Check if config file exists
if not exist ".env" (
    echo Error: Config file .env not found!
    echo Please ensure .env file exists with correct configuration
    echo.
    pause
    exit /b 1
)

REM Check if data file exists
if not exist "data.json" (
    echo Error: Data file data.json not found!
    echo Please ensure data file exists
    echo.
    pause
    exit /b 1
)

echo All required files checked successfully
echo.
echo Choose running mode:
echo 1. Send data once (default)
echo 2. Send every 5 seconds, 5 rounds
echo 3. Send every 10 seconds, infinite loop
echo 4. Test with example data file
echo 5. Custom parameters
echo.
set /p choice=Please enter choice (1-5):

REM Remove trailing spaces from choice
set choice=%choice: =%

if "%choice%"=="1" goto option1
if "%choice%"=="2" goto option2
if "%choice%"=="3" goto option3
if "%choice%"=="4" goto option4
if "%choice%"=="5" goto option5
goto option1

:option1
echo.
echo Starting to send data...
target\release\push-message-thingsboard.exe
goto end

:option2
echo.
echo Starting to send data every 5 seconds, 5 rounds...
target\release\push-message-thingsboard.exe --interval 5 --count 5
goto end

:option3
echo.
echo Starting infinite loop, send every 10 seconds...
echo Press Ctrl+C to stop
target\release\push-message-thingsboard.exe --interval 10 --count 0
goto end

:option4
if not exist "data_example.json" (
    echo Error: Example data file data_example.json not found!
    pause
    exit /b 1
)
echo.
echo Testing with example data file...
target\release\push-message-thingsboard.exe --file data_example.json --interval 2
goto end

:option5
echo.
set /p interval=Enter interval seconds:
set /p count=Enter rounds (0=infinite):
set /p file=Enter data file name (press enter for default):

if "%file%"=="" (
    set file=data.json
)

echo.
echo Starting to send data...
target\release\push-message-thingsboard.exe --interval %interval% --count %count% --file %file%
goto end

:end

echo.
echo ==========================================
echo Data sending completed!
echo ==========================================
pause
