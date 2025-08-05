@echo off
echo 正在构建 ThingsBoard 数据推送工具...
echo.

REM 构建发布版本
cargo build --release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ✅ 构建成功！
    echo.
    echo 可执行文件位置: target\release\push-message-thingsboard.exe
    echo.
    echo 使用方法:
    echo   .\target\release\push-message-thingsboard.exe --help
    echo.
) else (
    echo.
    echo ❌ 构建失败！
    echo.
)

pause
