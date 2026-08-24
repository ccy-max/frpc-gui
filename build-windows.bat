@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo ========================================
echo   FRPC GUI - Windows 构建脚本
echo ========================================
echo.

REM 检查 Node.js
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未检测到 Node.js，请先安装：https://nodejs.org/
    pause
    exit /b 1
)

echo [✓] Node.js: 
node --version
echo.

REM 检查 Rust
where rustc >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未检测到 Rust，请先安装：https://rustup.rs/
    pause
    exit /b 1
)

echo [✓] Rust: 
rustc --version
echo.

REM 检查 Visual Studio Build Tools
where cl.exe >nul 2>&1
if %errorlevel% neq 0 (
    echo [警告] 未检测到 Visual Studio Build Tools
    echo 请安装 Visual Studio Build Tools 2022:
    echo https://visualstudio.microsoft.com/visual-cpp-build-tools/
    echo.
    echo 正在尝试继续构建...
    echo.
) else (
    echo [✓] Visual Studio Build Tools 已安装
    echo.
)

REM 进入项目目录
cd /d %~dp0

REM 安装依赖
echo ========================================
echo 步骤 1/2: 安装 Node.js 依赖...
echo ========================================
call npm install
if %errorlevel% neq 0 (
    echo [错误] 依赖安装失败
    pause
    exit /b 1
)
echo.

REM 构建
echo ========================================
echo 步骤 2/2: 构建 Windows 版本...
echo ========================================
echo 这可能需要 5-10 分钟...
echo.

call npm run tauri build

if %errorlevel% neq 0 (
    echo.
    echo [错误] 构建失败
    echo 请检查上方的错误信息
    pause
    exit /b 1
)

echo.
echo ========================================
echo   构建成功！
echo ========================================
echo.
echo 生成的文件：
echo.
echo 1. 直接运行的 EXE:
echo    src-tauri\target\release\FRPC GUI.exe
echo.
echo 2. NSIS 安装程序:
echo    src-tauri\target\release\bundle\nsis\FRPC GUI_0.1.0_x64-setup.exe
echo.
echo 3. MSI 安装包:
echo    src-tauri\target\release\bundle\msi\FRPC GUI_0.1.0_x64.msi
echo.

dir src-tauri\target\release\FRPC*.exe 2>nul
echo.

pause
