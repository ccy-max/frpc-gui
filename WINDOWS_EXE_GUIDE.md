# FRPC GUI - 快速构建 Windows EXE 指南

## 🚀 最简单的方法

**如果你有 Windows 电脑**，只需 3 步：

### 1️⃣ 在 Windows 上安装必要工具

打开 PowerShell（管理员），运行以下命令：

```powershell
# 1. 安装 Rust
winget install --id Rustlang.Rustup

# 2. 安装 Visual Studio Build Tools
winget install --id Microsoft.VisualStudio.2022.BuildTools

# 3. 安装 Node.js (如果还没有)
winget install --id OpenJS.NodeJS.LTS
```

安装完成后，**重启电脑**。

### 2️⃣ 复制项目到 Windows

将整个 `frpc-gui` 文件夹复制到 Windows 电脑。

### 3️⃣ 双击运行构建脚本

在 Windows 上，双击运行：
```
frpc-gui\build-windows.bat
```

等待 5-10 分钟，构建完成后会在以下位置生成 EXE：

```
src-tauri\target\release\FRPC GUI.exe  ← 直接双击运行！
```

---

## 📦 构建产物说明

构建完成后，你会得到 3 个文件：

| 文件 | 说明 | 大小 |
|------|------|------|
| `FRPC GUI.exe` | **直接运行的可执行文件** | ~15 MB |
| `FRPC GUI_0.1.0_x64-setup.exe` | NSIS 安装程序 | ~16 MB |
| `FRPC GUI_0.1.0_x64.msi` | MSI 安装包 | ~15 MB |

**推荐使用 `FRPC GUI.exe`** - 双击即可运行，无需安装！

---

## 🤔 当前 Linux 环境怎么办？

当前运行环境是 **Linux**，无法直接构建 Windows EXE。

### 方案 A：使用 Windows 电脑（推荐）

1. 将项目复制到 Windows 电脑
2. 按照上述步骤构建
3. 将生成的 EXE 复制回 Linux 或直接使用

### 方案 B：使用 GitHub Actions 自动构建

我可以帮你配置 GitHub Actions，每次 push 代码后自动构建 Windows EXE。

### 方案 C：使用虚拟机

在 Linux 上安装 Windows 虚拟机，在虚拟机中构建。

---

## 📝 手动构建命令（如果在 Windows 上）

```powershell
# 进入项目目录
cd frpc-gui

# 安装依赖
npm install

# 构建
npm run tauri build

# 生成的文件位置
.\src-tauri\target\release\FRPC GUI.exe
```

---

## ❓ 常见问题

### Q: 我没有 Windows 电脑，怎么办？

A: 你可以：
1. 使用 GitHub Actions 云构建（我可以帮你配置）
2. 在 Linux 上先用 DEB/AppImage 版本测试
3. 使用 Wine 运行 Windows 版本（不推荐，兼容性差）

### Q: 构建需要多长时间？

A: 第一次构建约 5-10 分钟（需要下载依赖和编译）。后续构建只需 1-2 分钟。

### Q: EXE 文件有多大？

A: 约 15-20 MB，比 Electron 方案（150MB+）小得多。

### Q: 可以在没有安装 .NET 的电脑上运行吗？

A: 可以！Tauri 应用不需要 .NET Framework，只需要 WebView2（Windows 10/11 已预装）。

---

## 📞 需要帮助？

如果你在构建过程中遇到问题，请告诉我：
1. 你的 Windows 版本
2. 具体的错误信息
3. 构建日志

我会帮你解决！

---

**下一步建议**：
1. 在 Windows 电脑上构建 EXE
2. 或者告诉我，我帮你配置 GitHub Actions 自动构建
3. 或者先在 Linux 上测试 DEB/AppImage 版本
