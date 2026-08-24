# Windows 构建指南

由于 Tauri v2 的 Windows 构建需要在 Windows 环境下进行，请按照以下步骤在 Windows 机器上构建 EXE 文件。

## 环境要求

### 1. 安装 Visual Studio Build Tools

下载并安装 [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

安装时勾选：
- ✅ **C++ 生成工具** (C++ build tools)
- ✅ **MSVC v143 - VS 2022 C++ x64/x86 生成工具**
- ✅ **Windows 10/11 SDK**
- ✅ **C++ CMake 工具**

### 2. 安装 Rust

打开 PowerShell（管理员），运行：

```powershell
winget install --id Rustlang.Rustup
```

或者下载安装程序：https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe

安装后验证：

```powershell
rustc --version
cargo --version
```

### 3. 安装 Node.js

下载并安装 LTS 版本：https://nodejs.org/

验证安装：

```powershell
node --version
npm --version
```

### 4. 安装 WebView2

Windows 10/11 通常已预装 WebView2。如果没有，请下载：
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

## 构建步骤

### 1. 克隆或复制项目

将 `frpc-gui` 项目复制到 Windows 机器上。

### 2. 安装依赖

```powershell
cd frpc-gui
npm install
```

### 3. 构建 EXE

```powershell
npm run tauri build
```

构建完成后，产物位于：

```
src-tauri/target/release/
├── FRPC GUI.exe              # 直接运行的可执行文件
└── bundle/
    ├── msi/
    │   └── FRPC GUI_0.1.0_x64.msi          # MSI 安装包
    └── nsis/
        └── FRPC GUI_0.1.0_x64-setup.exe    # NSIS 安装程序
```

## 直接使用

### 方法 1: 直接运行 EXE

```powershell
cd src-tauri/target/release
.\FRPC GUI.exe
```

### 方法 2: 创建便携版

1. 复制整个 `release` 文件夹
2. 包含以下文件：
   - `FRPC GUI.exe`
   - `FRPC GUI.exe.config` (如果有)
   - 所有 `.dll` 文件
   - `resources` 文件夹 (如果有)

3. 压缩成 ZIP 即可作为便携版分发

## 常见问题

### Q: 构建时提示找不到 WebView2？

A: 安装 WebView2 运行时：
```powershell
winget install Microsoft.WebView2
```

### Q: 构建失败，提示链接错误？

A: 确保 Visual Studio Build Tools 安装完整，并重启 PowerShell。

### Q: 如何构建 32 位版本？

A: 修改 `src-tauri/tauri.conf.json`：
```json
{
  "bundle": {
    "targets": ["msi", "nsis", "updater"],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "",
      "wix": null,
      "nsis": null
    }
  }
}
```

然后运行：
```powershell
rustup target add i686-pc-windows-msvc
npm run tauri build -- --target i686-pc-windows-msvc
```

## 自动化脚本

创建 `build-windows.bat`：

```batch
@echo off
echo Building FRPC GUI for Windows...

cd /d %~dp0

echo Installing dependencies...
call npm install

echo Building release version...
call npm run tauri build

echo.
echo Build completed!
echo Executable: src-tauri\target\release\FRPC GUI.exe
echo Installer: src-tauri\target\release\bundle\nsis\FRPC GUI_0.1.0_x64-setup.exe

pause
```

双击运行即可自动构建。

---

**注意**: 当前 Linux 环境下无法直接构建 Windows EXE，必须在 Windows 机器上执行上述步骤。
