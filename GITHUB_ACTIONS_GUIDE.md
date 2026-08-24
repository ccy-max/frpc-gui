# GitHub Actions 自动构建指南

## 🚀 配置完成！

GitHub Actions 工作流已配置在：
```
.github/workflows/build.yml
```

## 📋 使用方法

### 方法 1：推送到 GitHub 自动构建

1. **初始化 Git 仓库**（如果还没有）：
   ```bash
   cd frpc-gui
   git init
   git add .
   git commit -m "Initial commit: FRPC GUI"
   ```

2. **创建 GitHub 仓库**：
   - 访问 https://github.com/new
   - 创建名为 `frpc-gui` 的新仓库
   - 不要勾选 "Initialize this repository with a README"

3. **推送代码到 GitHub**：
   ```bash
   git remote add origin https://github.com/YOUR_USERNAME/frpc-gui.git
   git branch -M main
   git push -u origin main
   ```

4. **等待构建完成**：
   - 访问 https://github.com/YOUR_USERNAME/frpc-gui/actions
   - 等待 10-15 分钟
   - 构建完成后，在 **Artifacts** 部分下载 EXE 文件

### 方法 2：手动触发构建

1. 推送到 GitHub 后
2. 访问 https://github.com/YOUR_USERNAME/frpc-gui/actions/workflows/build.yml
3. 点击右上角的 **"Run workflow"** 按钮
4. 选择分支（通常是 `main`）
5. 点击 **"Run workflow"**
6. 等待构建完成并下载

### 方法 3：创建 Release 自动发布

1. **打标签**：
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **自动创建 Release**：
   - Actions 会自动构建所有平台
   - 在 https://github.com/YOUR_USERNAME/frpc-gui/releases 查看
   - 构建产物会自动上传到 Release

## 📦 构建产物

### Windows
- ✅ `FRPC GUI.exe` - 直接运行的可执行文件（~15MB）
- ✅ `FRPC GUI_*_x64-setup.exe` - NSIS 安装程序
- ✅ `FRPC GUI_*_x64.msi` - MSI 安装包

### Linux
- ✅ `FRPC GUI_*.amd64.deb` - Debian/Ubuntu 包
- ✅ `FRPC GUI-*.x86_64.rpm` - RedHat/Fedora 包
- ✅ `FRPC GUI_*.amd64.AppImage` - 通用 AppImage

### macOS
- ✅ `FRPC GUI_*.dmg` - 磁盘镜像
- ✅ `FRPC GUI.app` - 应用程序包

## 📥 下载构建产物

### 从 Actions 下载（未发布时）

1. 访问 https://github.com/YOUR_USERNAME/frpc-gui/actions
2. 点击最近的构建（例如 "Build Windows EXE #1"）
3. 在页面底部找到 **Artifacts** 部分
4. 点击 `FRPC-GUI-Windows-EXE` 下载
5. 解压 ZIP 文件，得到 `FRPC GUI.exe`

### 从 Releases 下载（发布后）

1. 访问 https://github.com/YOUR_USERNAME/frpc-gui/releases
2. 选择版本（例如 v0.1.0）
3. 在 **Assets** 部分下载需要的文件
4. 直接运行 `FRPC GUI.exe`

## ⚙️ 自定义配置

### 只构建 Windows

编辑 `.github/workflows/build.yml`，注释掉其他平台：

```yaml
jobs:
  build-windows:
    # ... 保持不变
  
  # build-linux:
  #   ... 注释掉
  
  # build-macos:
  #   ... 注释掉
```

### 只构建特定平台

删除不需要的 `jobs` 部分即可。

### 修改触发条件

```yaml
on:
  push:
    branches: [main]  # 只在 main 分支触发
    tags: ['v*']      # 打标签时触发
  # 移除 workflow_dispatch 禁用手动触发
```

## 🔧 故障排除

### Q: Actions 没有运行？

A: 检查：
1. 仓库的 **Settings** → **Actions** → **General**
2. 确保 **Allow all actions** 已启用
3. 检查是否有足够的 GitHub Actions 配额

### Q: 构建失败？

A: 查看构建日志：
1. 访问 Actions 页面
2. 点击失败的构建
3. 展开日志查看错误信息
4. 常见错误：
   - 依赖安装失败 → 检查网络连接
   - Rust 编译错误 → 检查代码
   - 内存不足 → 使用更大规格的 runner

### Q: 下载链接过期？

A: Artifacts 保留 30 天，过期后：
1. 重新触发构建
2. 或创建 Release（永久保留）

## 📊 GitHub Actions 配额

免费账户每月有 **2000 分钟** 的构建时间：
- Windows 构建：~15 分钟/次
- Linux 构建：~10 分钟/次
- macOS 构建：~20 分钟/次

大约可以构建 **100+ 次/月**

## 🎯 下一步

1. **推送代码到 GitHub**
2. **触发第一次构建**
3. **下载 EXE 文件测试**
4. **创建第一个 Release**

---

需要帮助？查看：
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Tauri 部署指南](https://v2.tauri.app/distribute/ci/)

⟦ FRPC GUI - GitHub Actions 配置完成 ⟧
