# FRPC GUI

FRP 内网穿透桌面管理应用 - 基于 Tauri v2 + Vue 3 开发

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB.svg)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-v3-4FC08D.svg)](https://vuejs.org/)
[![Build](https://img.shields.io/github/actions/workflow/status/your-username/frpc-gui/build.yml?label=Build)](https://github.com/your-username/frpc-gui/actions)

## 📖 项目简介

FRPC GUI 是一款跨平台的 FRP 内网穿透桌面管理工具，完全对标 [frpc-desktop](https://github.com/luckjiawei/frpc-desktop)，采用 Rust + Tauri v2 + Vue 3 技术栈开发。

相比 Electron 方案，本应用具有：
- 🚀 **更小的体积** - 打包后仅 10-20MB（Electron 约 150MB+）
- ⚡ **更快的性能** - 原生编译，内存占用低
- 🔒 **更高的安全性** - Rust 内存安全，类型安全
- 💻 **真正的跨平台** - Windows/macOS/Linux 原生支持

## ✨ 功能特性

### 核心功能
- ✅ 可视化配置 FRP 客户端（服务器地址、令牌、端口等）
- ✅ 支持所有 frp 版本（自动下载/管理 frpc 二进制）
- ✅ 开机自启动
- ✅ 实时日志查看与控制台输出
- ✅ 多代理管理（TCP / UDP / HTTP / HTTPS / STCP / XTCP）
- ✅ 批量端口配置
- ✅ 配置导入导出（JSON 格式）
- ✅ 多语言支持（中英文）
- ✅ 系统托盘图标与右键菜单
- ✅ 全局快捷键支持

### 支持的代理类型
- **TCP** - TCP 端口映射
- **UDP** - UDP 端口映射
- **HTTP** - HTTP 服务映射
- **HTTPS** - HTTPS 服务映射
- **STCP** - 秘密隧道 TCP
- **XTCP** - P2P 隧道 TCP

## 🛠️ 技术栈

### 前端
- **框架**: Vue 3.5 + TypeScript
- **UI 组件**: Element Plus
- **状态管理**: Pinia
- **路由**: Vue Router
- **国际化**: Vue I18n
- **构建工具**: Vite 6

### 后端
- **语言**: Rust 2021
- **桌面框架**: Tauri v2
- **异步运行时**: Tokio
- **序列化**: Serde + Serde JSON
- **HTTP 客户端**: Reqwest
- **配置解析**: Toml
- **进程管理**: Sysinfo
- **压缩**: Zip + Tar + Flate2

## 📦 项目结构

```
frpc-gui/
├── src/                          # 前端源代码
│   ├── components/               # Vue 组件
│   │   └── Layout.vue           # 主布局组件
│   ├── views/                    # 页面视图
│   │   ├── Dashboard.vue        # 概览页面
│   │   ├── Servers.vue          # 服务器管理
│   │   ├── Proxies.vue          # 代理管理
│   │   ├── Versions.vue         # 版本管理
│   │   ├── Logs.vue             # 日志查看
│   │   ├── Settings.vue         # 设置页面
│   │   └── About.vue            # 关于页面
│   ├── stores/                   # Pinia 状态管理
│   │   └── app.ts               # 应用状态
│   ├── i18n/                     # 国际化配置
│   │   ├── index.ts
│   │   └── locales.ts           # 语言包
│   ├── types/                    # TypeScript 类型定义
│   │   └── index.ts             # FRP 相关类型
│   ├── utils/                    # 工具函数
│   ├── styles/                   # 样式文件
│   │   ├── variables.scss       # SCSS 变量
│   │   └── global.scss          # 全局样式
│   ├── router/                   # 路由配置
│   │   └── index.ts
│   ├── App.vue                   # 根组件
│   └── main.ts                   # 入口文件
├── src-tauri/                    # Tauri/Rust 后端
│   ├── src/
│   │   ├── main.rs              # Rust 入口
│   │   └── lib.rs               # Tauri 命令模块
│   ├── icons/                    # 应用图标
│   ├── Cargo.toml               # Rust 依赖
│   ├── tauri.conf.json          # Tauri 配置
│   └── build.rs                 # 构建脚本
├── package.json                  # Node.js 依赖
├── tsconfig.json                 # TypeScript 配置
├── vite.config.ts               # Vite 配置
└── README.md                     # 项目说明

```

## 🚀 快速开始

### 环境要求

- **Node.js**: >= 18.0.0
- **Rust**: >= 1.70.0
- **系统依赖** (Linux):
  ```bash
  # Ubuntu/Debian
  sudo apt install libwebkit2gtk-4.1-dev librsvg2-dev libgtk-3-dev \
                   libayatana-appindicator3-dev libjavascriptcoregtk-4.1-dev \
                   libsoup-3.0-dev libxdo-dev libssl-dev pkg-config \
                   libclang-dev cmake build-essential
  
  # Arch Linux
  sudo pacman -S webkit2gtk-4.1 rsvg2 libgtk-3 libappindicator-gtk3 \
                 libxdo openssl pkgconfig clang cmake
  ```

### 安装依赖

```bash
# 安装 Node.js 依赖
npm install

# 验证 Rust 环境
rustc --version
cargo --version
```

### 开发模式

```bash
# 启动开发服务器（前端 + Tauri）
npm run tauri dev
```

### 构建发布

```bash
# 构建生产版本
npm run tauri build

# 构建产物位置：
# - Windows: src-tauri/target/release/bundle/msi/ 和 nsis/
# - macOS: src-tauri/target/release/bundle/dmg/ 和 app/
# - Linux: src-tauri/target/release/bundle/deb/ 和 appimage/
```

## 📥 获取 Windows EXE

### 方法 1：GitHub Actions 自动构建（推荐）

1. **推送代码到 GitHub**：
   ```bash
   ./setup-github-actions.sh  # 运行设置脚本
   git remote add origin https://github.com/YOUR_USERNAME/frpc-gui.git
   git push -u origin main
   ```

2. **等待构建完成**：
   - 访问 https://github.com/YOUR_USERNAME/frpc-gui/actions
   - 等待 10-15 分钟
   - 在 **Artifacts** 下载 `FRPC-GUI-Windows-EXE`

3. **运行 EXE**：
   - 解压 ZIP 文件
   - 双击 `FRPC GUI.exe` 即可运行

### 方法 2：在 Windows 上手动构建

详见 [BUILD_WINDOWS.md](BUILD_WINDOWS.md)

### 方法 3：查看完整指南

详见 [GITHUB_ACTIONS_GUIDE.md](GITHUB_ACTIONS_GUIDE.md)

## 📝 开发计划

### PHASE 1: 项目搭建 ✅
- [x] Tauri v2 + Vue 3 项目初始化
- [x] 基础 UI 框架搭建
- [x] 路由和状态管理配置
- [x] 国际化支持
- [x] 基础视图组件开发

### PHASE 2: 核心功能开发 🚧
- [ ] FRP 二进制文件管理（下载、版本检测、删除）
- [ ] FRP 配置文件解析与生成（TOML/INI 格式）
- [ ] FRP 进程管理（启动、停止、重启、状态监控）
- [ ] 实时日志捕获与展示
- [ ] Rust 后端命令实现

### PHASE 3: 增强功能开发
- [ ] 开机自启动配置
- [ ] 配置导入导出（JSON 格式）
- [ ] 多代理管理界面优化
- [ ] 批量端口配置
- [ ] 系统托盘图标与右键菜单
- [ ] 全局快捷键支持

### PHASE 4: 国际化与打包
- [ ] 完善中英文翻译
- [ ] 跨平台打包测试
- [ ] 自动更新机制
- [ ] 性能优化与测试

## 🔧 配置说明

### Tauri 配置 (tauri.conf.json)

```json
{
  "productName": "FRPC GUI",
  "version": "0.1.0",
  "identifier": "com.comdustinkyqwenpaw.frpc-gui",
  "app": {
    "windows": [{
      "title": "FRPC GUI - FRP 内网穿透管理",
      "width": 1200,
      "height": 800
    }],
    "trayIcon": {
      "iconPath": "icons/32x32.png"
    }
  }
}
```

### FRP 配置格式

应用支持标准的 frpc.toml 格式：

```toml
serverAddr = "127.0.0.1"
serverPort = 7000
auth.method = "token"
auth.token = "your_token"

[[proxies]]
name = "web"
type = "tcp"
localIP = "127.0.0.1"
localPort = 8080
remotePort = 80
```

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 开源协议

MIT License - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- [frp](https://github.com/fatedier/frp) - 高性能的内网穿透应用
- [frpc-desktop](https://github.com/luckjiawei/frpc-desktop) - 优秀的 Electron 版本参考
- [Tauri](https://tauri.app/) - 现代化的桌面应用框架
- [Vue 3](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Element Plus](https://element-plus.org/) - Vue 3 组件库

## 📬 联系方式

- **作者**: Zero
- **Email**: zero@example.com
- **GitHub**: [your-repo/frpc-gui](https://github.com/your-repo/frpc-gui)

---

**注意**: 当前版本为 Alpha 开发版，部分功能尚未完成。请在生产环境使用前进行充分测试。
