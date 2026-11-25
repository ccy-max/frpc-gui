# FRPC GUI

FRP 内网穿透桌面管理应用 - 基于 Tauri v2 + Vue 3 开发

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB.svg)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-v3-4FC08D.svg)](https://vuejs.org/)
[![Build](https://img.shields.io/github/actions/workflow/status/ccy-max/frpc-gui/build-windows.yml?label=Build)](https://github.com/ccy-max/frpc-gui/actions)
[![Version](https://img.shields.io/badge/Version-v1.0.17-blue.svg)](https://github.com/ccy-max/frpc-gui/releases)

## 📖 项目简介

FRPC GUI 是一款跨平台的 FRP 内网穿透桌面管理工具，采用 Rust + Tauri v2 + Vue 3 技术栈开发。

相比 Electron 方案，本应用具有：
- 🚀 **更小的体积** - 打包后约 10-20MB（Electron 约 150MB+）
- ⚡ **更快的性能** - 原生编译，内存占用低
- 🔒 **更高的安全性** - Rust 内存安全，类型安全
- 💻 **真正的跨平台** - Windows/macOS/Linux 原生支持

## ✨ 功能特性

### 核心功能
- ✅ **多服务器管理** - 每个服务器独立 frpc 进程，互不干扰
- ✅ **代理管理** - 支持 TCP / UDP / HTTP / HTTPS / STCP / XTCP 全类型
- ✅ **版本管理** - 自动下载/导入/切换 frpc 二进制（v0.52+ TOML / 旧版 INI 双格式）
- ✅ **实时监控** - 概览页代理请求状态（在线/离线/流量），Admin API 驱动
- ✅ **流量统计** - 按服务器/代理维度的流量数据 + 历史趋势
- ✅ **日志查看** - 磁盘日志持久化，按需加载历史记录
- ✅ **系统托盘** - 最小化到托盘，右键菜单快捷操作
- ✅ **开机自启** - 可选开机自动启动
- ✅ **配置导入导出** - JSON 格式备份/恢复
- ✅ **多语言支持** - 中文 / 英文
- ✅ **主题切换** - 亮色 / 暗色 / 跟随系统

### 增强功能
- ✅ **流量统计页** - ECharts 流量趋势 / 代理 Top10 / 连接事件时间线
- ✅ **HTTP 请求检查器** - 本地反代 + 环形缓冲 200 条 + 一键重放（对标 ngrok）
- ✅ **命令面板** - `Ctrl/Cmd+K` 快捷跳转 / 启停服务器 / 新建 / 调试
- ✅ **桌面通知** - 掉线 / 恢复 / 启动失败提醒（设置页可开关）
- ✅ **批量端口创建** - TCP / UDP 连续端口一次建 N 条
- ✅ **多用户字段** - `user` + `meta_token` 支持 frps multiuser
- ✅ **双视图切换** - 代理卡片 / 列表视图随心切换
- ✅ **进程级流量监控** - Windows `GetProcessIoCounters` 精确采集

### 性能优化
- 🚀 **frpc 版本缓存** - `once_cell` 缓存，切换版本失效，省 0.3~1s/次
- 🚀 **进程内枚举** - `sysinfo` 替代 PowerShell/tasklist 子进程，省 1~3s/次
- 🚀 **路由懒加载** - 页面独立 chunk，首屏 JS 大幅缩减
- 🚀 **轮询并行化** - 多命令并行 + 防堆积守卫，启停乐观更新防连点

### 概览页
- 运行状态卡片（运行中/已停止 + 运行时长）
- 代理总数 / 活跃代理统计
- 快捷操作（启动/停止 FRP、添加服务器、查看日志、设置）
- 代理请求信息卡片（Admin API 轮询，5 秒自动刷新）
- 首屏骨架屏 + 呼吸灯状态动画

### 服务器管理
- 服务器卡片式布局
- 默认服务器设置（概览页快捷操作的目标）
- 每服务器独立配置目录与日志
- 动态 Admin API 端口分配（避免冲突）

## 🛠️ 技术栈

### 前端
- **框架**: Vue 3.5 + TypeScript
- **UI 组件**: Ant Design Vue 4
- **状态管理**: Pinia
- **路由**: Vue Router
- **构建工具**: Vite 6

### 后端
- **语言**: Rust 2021
- **桌面框架**: Tauri v2
- **异步运行时**: Tokio
- **序列化**: Serde + Serde JSON
- **HTTP 客户端**: Reqwest
- **配置解析**: Toml
- **日志**: Log + Env_logger
- **压缩**: Zip + Tar + Flate2

## 📦 项目结构

```
frpc-gui/
├── src/                          # 前端源代码
│   ├── views/                    # 页面视图
│   │   ├── Dashboard.vue         # 概览（状态/代理请求/快捷操作）
│   │   ├── Servers.vue           # 服务器管理（卡片布局）
│   │   ├── ServerEdit.vue        # 服务器新建/编辑（页内页面）
│   │   ├── Proxies.vue           # 代理管理（多类型 + 双视图）
│   │   ├── ProxyEdit.vue         # 代理新建/编辑（页内页面）
│   │   ├── Versions.vue          # 版本管理（下载/切换）
│   │   ├── Logs.vue              # 日志查看（虚拟滚动）
│   │   ├── Stats.vue             # 流量统计（ECharts 趋势/Top10/时间线）
│   │   ├── Inspector.vue         # HTTP 请求检查器（重放）
│   │   ├── Settings.vue          # 设置（默认服务器/主题/语言/通知）
│   │   └── About.vue             # 关于
│   ├── stores/                   # Pinia 状态管理
│   │   └── app.ts                # 全局状态 + 轮询
│   ├── types/                    # TypeScript 类型定义
│   ├── router/                   # 路由配置（懒加载）
│   ├── App.vue                   # 根组件（防刷新拦截）
│   └── main.ts                   # 入口文件
├── src-tauri/                    # Tauri/Rust 后端
│   ├── src/
│   │   ├── lib.rs                # 应用入口（退出清理/日志通道）
│   │   ├── main.rs               # Rust main
│   │   ├── commands/mod.rs       # Tauri 命令（start/stop/status/traffic/admin）
│   │   ├── utils/                # 工具模块
│   │   │   ├── mod.rs
│   │   │   └── settings.rs       # 应用设置读写
│   │   └── frp/                  # FRP 核心模块
│   │       ├── config.rs         # 配置生成（TOML/INI 双格式）
│   │       ├── process.rs        # 进程管理（spawn/kill/流量/orphan清理）
│   │       ├── download.rs       # 版本下载（镜像回退链）
│   │       └── inspector.rs      # HTTP 请求检查器（环形缓冲/重放）
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 配置
│   └── build.rs                 # 构建脚本
├── .github/workflows/           # GitHub Actions CI
├── package.json                  # Node.js 依赖
└── vite.config.ts               # Vite 配置
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
  ```

### 安装与开发

```bash
# 安装前端依赖
npm install

# 开发模式（前端热更新 + Tauri 窗口）
npm run tauri dev

# 构建生产版本
npm run tauri build
```

## 📥 获取 Windows 安装包

### GitHub Actions 自动构建（推荐）

1. 访问 [Releases 页面](https://github.com/ccy-max/frpc-gui/releases)
2. 下载最新版本的 `frpc-gui_x64-setup.exe`（NSIS 安装包）
3. 双击安装即可使用

### 从源码构建

```bash
git clone https://github.com/ccy-max/frpc-gui.git
cd frpc-gui
npm install
npm run tauri build
```

## 🔧 配置说明

### 应用配置

应用数据存储在用户配置目录下：
- **Windows**: `%APPDATA%/frpc-gui/`
- **Linux**: `~/.config/frpc-gui/`

目录结构：
```
frpc-gui/
├── settings.json          # 应用设置（主题/语言/默认服务器）
├── frpc-gui-data.json     # 持久化数据（服务器/代理/版本）
├── frpc.log               # 全局日志文件
├── bin/                   # frpc 二进制文件
│   ├── frpc.exe
│   └── versions/          # 多版本
└── servers/               # 每服务器独立配置
    └── {server_id}/
        ├── config.toml    # 生成的 frpc 配置
        └── frpc.log       # 服务器日志
```

### FRP 配置格式

应用自动生成 frpc 配置（新版 TOML / 旧版 INI 自动适配）：

```toml
serverAddr = "your-server.com"
serverPort = 7000
loginFailExit = false

[auth]
method = "token"
token = "your_token"

[log]
level = "info"
maxDays = 7

[webServer]
addr = "127.0.0.1"
port = 7400

[[proxies]]
name = "web"
type = "tcp"
localIP = "127.0.0.1"
localPort = "8080"
remotePort = 9090
```

## 📝 开发历程

### 已完成

- ✅ **v0.1.0** - 项目搭建：Tauri v2 + Vue 3 + Ant Design Vue
- ✅ **v1.0.0** - 核心功能：配置管理、进程控制、版本下载
- ✅ **v1.0.1-v1.0.6** - Bug 修复：配置生成、日志显示、进程清理
- ✅ **v1.0.7-v1.0.8** - 格式兼容：TOML/INI 双格式、旧版 frpc 支持
- ✅ **v1.0.9-v1.0.10** - 下载修复：镜像回退链、版本切换
- ✅ **v1.0.11-v1.0.12** - 日志/持久化：磁盘日志预填、孤儿进程三级查找
- ✅ **v1.0.13** - 全量代码审查修复（33 项发现：死锁/契约/监控/镜像）
- ✅ **v1.0.14-v1.0.15** - CI 修复 + 启动回归修复
- ✅ **v1.0.16** - 概览页状态同步 + 加载动画 + 日志改代理请求
- ✅ **v1.0.17** - 代理请求信息卡片 + 右键刷新防护 + 仓库清理
- ✅ **v1.0.18-v1.0.19** - CI 完善：exe 产物带版本号、自动创建 Release
- ✅ **v1.0.20-v1.0.21** - 关闭窗口白屏/死锁修复，改用 `exit_app` 命令
- ✅ **v1.0.22-v1.0.25** - 概览页代理请求卡片兼容空 `server_id`、状态与日志显示修复
- ✅ **v1.0.26-v1.0.29** - 调试支持：devtools、F12、调试控制台按钮
- ✅ **v1.0.30-v1.0.31** - 概览页崩溃根因修复 + 全面体检
- ✅ **v1.1.0** - 全量增强（26 项）：性能（版本缓存/sysinfo/懒加载/轮询并行）+ UI/UX（命令面板/设计 Token/骨架屏/呼吸灯）+ 新功能（统计页/HTTP 检查器/桌面通知/批量端口/多用户/双视图/i18n）
- ✅ **v1.1.1-v1.1.3** - 版本号同步 + Windows 进程级流量监控（`GetProcessIoCounters`）
- ✅ **v1.1.4-v1.1.5** - 统计页刷新/空状态修复 + 关闭残留 frpc 进程修复
- ✅ **v1.1.6-v1.1.7** - 侧边栏折叠错位修复 + 卡片紧凑风格
- ✅ **v1.1.8-v1.1.9** - 新建/编辑改页内页面 + 子页面头部丢失修复

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
- [Tauri](https://tauri.app/) - 现代化的桌面应用框架
- [Vue 3](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Ant Design Vue](https://antdv.com/) - 企业级 Vue 3 组件库
