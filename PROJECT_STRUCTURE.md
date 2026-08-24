# FRPC GUI 项目结构说明

## 完整目录树

```
frpc-gui/
│
├── 📄 README.md                      # 项目说明文档
├── 📄 DEVELOPMENT.md                 # 开发指南
├── 📄 FRPC_GUI_TODO.md               # 开发任务清单 (workspace 根目录)
├── 📄 package.json                   # Node.js 依赖配置
├── 📄 package-lock.json              # 依赖锁定文件
├── 📄 tsconfig.json                  # TypeScript 配置
├── 📄 tsconfig.node.json             # Node.js TypeScript 配置
├── 📄 vite.config.ts                 # Vite 构建配置
├── 📄 .gitignore                     # Git 忽略规则
│
├── 📁 .vscode/                       # VSCode 配置
│   └── extensions.json               # 推荐扩展
│
├── 📁 src/                           # 前端源代码目录
│   │
│   ├── 📄 main.ts                    # Vue 应用入口
│   ├── 📄 App.vue                    # 根组件
│   ├── 📄 vite-env.d.ts              # Vite 环境类型定义
│   ├── 📄 auto-imports.d.ts          # 自动导入类型定义
│   ├── 📄 components.d.ts            # 组件类型定义
│   │
│   ├── 📁 components/                # Vue 组件
│   │   └── 📄 Layout.vue             # 主布局组件（侧边栏 + 内容区）
│   │
│   ├── 📁 views/                     # 页面视图组件
│   │   ├── 📄 Dashboard.vue          # 概览页面（统计卡片 + 快速操作）
│   │   ├── 📄 Servers.vue            # 服务器管理页面（CRUD 操作）
│   │   ├── 📄 Proxies.vue            # 代理管理页面（CRUD 操作）
│   │   ├── 📄 Versions.vue           # FRP 版本管理页面
│   │   ├── 📄 Logs.vue               # 日志查看页面
│   │   ├── 📄 Settings.vue           # 设置页面
│   │   └── 📄 About.vue              # 关于页面
│   │
│   ├── 📁 stores/                    # Pinia 状态管理
│   │   └── 📄 app.ts                 # 应用全局状态
│   │
│   ├── 📁 i18n/                      # 国际化配置
│   │   ├── 📄 index.ts               # i18n 实例配置
│   │   └── 📄 locales.ts             # 中英文语言包
│   │
│   ├── 📁 types/                     # TypeScript 类型定义
│   │   └── 📄 index.ts               # FRP 相关类型（ServerConfig, ProxyConfig 等）
│   │
│   ├── 📁 router/                    # 路由配置
│   │   └── 📄 index.ts               # Vue Router 配置
│   │
│   ├── 📁 utils/                     # 工具函数（待开发）
│   │
│   ├── 📁 styles/                    # 样式文件
│   │   ├── 📄 variables.scss         # SCSS 变量（颜色、间距、字体等）
│   │   └── 📄 global.scss            # 全局样式（滚动条、通用类等）
│   │
│   └── 📁 assets/                    # 静态资源（待添加）
│
├── 📁 src-tauri/                     # Tauri/Rust 后端目录
│   │
│   ├── 📄 Cargo.toml                 # Rust 依赖配置
│   ├── 📄 tauri.conf.json            # Tauri 应用配置
│   ├── 📄 build.rs                   # Rust 构建脚本
│   │
│   ├── 📁 src/                       # Rust 源代码
│   │   ├── 📄 main.rs                # Rust 入口（调用 lib.rs）
│   │   └── 📄 lib.rs                 # Tauri 命令模块（配置加载/保存）
│   │
│   ├── 📁 icons/                     # 应用图标（待添加）
│   │   ├── 32x32.png
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── icon.icns                 # macOS 图标
│   │   └── icon.ico                  # Windows 图标
│   │
│   ├── 📁 capabilities/              # Tauri 权限配置
│   │   └── default.json              # 默认权限
│   │
│   └── 📁 target/                    # 构建产物（由 Cargo 生成）
│       ├── release/                  # Release 构建
│       └── debug/                    # Debug 构建
│
└── 📁 dist/                          # 前端构建产物（由 Vite 生成）
    ├── index.html
    └── assets/
        ├── *.js
        └── *.css
```

## 核心文件说明

### 前端核心文件

| 文件 | 说明 | 行数 |
|------|------|------|
| `src/main.ts` | Vue 应用入口，注册插件和组件 | ~20 行 |
| `src/App.vue` | 根组件，加载路由 | ~15 行 |
| `src/components/Layout.vue` | 主布局，侧边栏导航 + 路由视图 | ~90 行 |
| `src/stores/app.ts` | Pinia 状态管理，应用配置和数据 | ~150 行 |
| `src/router/index.ts` | Vue Router 配置，7 个路由 | ~50 行 |
| `src/i18n/locales.ts` | 中英文语言包，完整翻译 | ~250 行 |
| `src/types/index.ts` | TypeScript 类型定义 | ~70 行 |

### 视图组件

| 组件 | 功能 | 行数 |
|------|------|------|
| `Dashboard.vue` | 概览页面，统计卡片 + 快速操作 + 最近日志 | ~150 行 |
| `Servers.vue` | 服务器管理，CRUD 操作 + 表单对话框 | ~260 行 |
| `Proxies.vue` | 代理管理，CRUD 操作 + 表单对话框 | ~200 行 |
| `Versions.vue` | FRP 版本管理，下载/删除 | ~70 行 |
| `Logs.vue` | 日志查看，表格展示 + 清空/导出 | ~80 行 |
| `Settings.vue` | 设置页面，通用设置 + 高级设置 | ~120 行 |
| `About.vue` | 关于页面，应用信息 + 链接 | ~70 行 |

### 后端核心文件

| 文件 | 说明 | 行数 |
|------|------|------|
| `src-tauri/src/main.rs` | Rust 入口，调用 lib.rs 的 run 函数 | ~5 行 |
| `src-tauri/src/lib.rs` | Tauri 命令定义，配置结构 | ~100 行 |
| `src-tauri/Cargo.toml` | Rust 依赖配置 | ~40 行 |
| `src-tauri/tauri.conf.json` | Tauri 应用配置 | ~60 行 |

## 技术栈版本

### 前端

```json
{
  "vue": "^3.5.13",
  "vue-router": "^4.2.5",
  "pinia": "^2.1.7",
  "vue-i18n": "^9.14.5",
  "element-plus": "^2.5.0",
  "@element-plus/icons-vue": "^2.3.1",
  "vite": "^6.0.3",
  "typescript": "~5.6.2",
  "@tauri-apps/api": "^2",
  "@tauri-apps/cli": "^2"
}
```

### 后端

```toml
[dependencies]
tauri = "2"
tauri-plugin-opener = "2"
tauri-plugin-shell = "2"
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-notification = "2"
tauri-plugin-autostart = "2"
serde = "1"
serde_json = "1"
tokio = "1"
anyhow = "1"
log = "0.4"
env_logger = "0.11"
reqwest = "0.11"
toml = "0.8"
dirs = "5"
sysinfo = "0.30"
zip = "0.6"
tar = "0.4"
flate2 = "1"
```

## 已实现功能

### ✅ PHASE 1: 项目搭建

1. **项目初始化**
   - Tauri v2 + Vue 3 + TypeScript 模板
   - Vite 6 构建配置
   - Element Plus UI 集成

2. **路由系统**
   - 7 个主要页面路由
   - 布局组件 + 路由视图
   - 导航菜单自动高亮

3. **状态管理**
   - Pinia Store 配置
   - 应用配置状态
   - 服务器/代理列表状态
   - 日志状态管理

4. **国际化**
   - 中英文完整翻译
   - 语言切换功能
   - 语言包结构化组织

5. **UI 组件**
   - 侧边栏导航布局
   - Dashboard 统计卡片
   - 表格 CRUD 界面
   - 表单对话框
   - 设置页面

6. **样式系统**
   - SCSS 变量定义
   - 全局样式
   - 滚动条美化
   - 响应式布局

## 待实现功能

### 🚧 PHASE 2: 核心功能

1. **FRP 二进制管理**
   - [ ] 版本检测 API
   - [ ] 下载管理器
   - [ ] 解压和安装
   - [ ] 版本列表展示

2. **配置管理**
   - [ ] TOML 解析器
   - [ ] 配置文件生成
   - [ ] 配置验证
   - [ ] 文件持久化

3. **进程管理**
   - [ ] 启动 FRP 进程
   - [ ] 停止 FRP 进程
   - [ ] 进程状态监控
   - [ ] 进程重启

4. **日志系统**
   - [ ] 实时日志捕获
   - [ ] 日志级别过滤
   - [ ] 日志导出
   - [ ] 日志文件轮转

### ⏳ PHASE 3: 增强功能

- [ ] 开机自启动
- [ ] 配置导入导出
- [ ] 批量端口配置
- [ ] 系统托盘
- [ ] 全局快捷键

### ⏳ PHASE 4: 打包发布

- [ ] Windows 打包（MSI/NSIS）
- [ ] macOS 打包（DMG/App）
- [ ] Linux 打包（DEB/AppImage）
- [ ] 自动更新

## 开发环境要求

### 必需

- Node.js >= 18.0.0
- Rust >= 1.70.0
- npm 或 yarn

### 系统依赖 (Linux)

```bash
sudo apt install libwebkit2gtk-4.1-dev librsvg2-dev libgtk-3-dev \
                 libayatana-appindicator3-dev libjavascriptcoregtk-4.1-dev \
                 libsoup-3.0-dev libxdo-dev libssl-dev pkg-config \
                 libclang-dev cmake build-essential
```

### 系统依赖 (macOS)

```bash
xcode-select --install
```

### 系统依赖 (Windows)

- Visual Studio Build Tools 2019+
- WebView2 (Windows 10 1803+ 自带)

## 构建命令

```bash
# 开发模式
npm run tauri dev

# 构建生产版本
npm run tauri build

# 仅构建前端
npm run build

# 仅前端开发
npm run dev
```

## 下一步开发建议

1. **优先实现后端核心逻辑**
   - FRP 下载和管理
   - 配置文件解析
   - 进程启动/停止

2. **完善前端交互**
   - 连接后端 API
   - 实时状态更新
   - 错误处理

3. **测试和优化**
   - 单元测试
   - 集成测试
   - 性能优化

4. **文档完善**
   - 用户手册
   - API 文档
   - 常见问题

---

**当前状态**: PHASE 1 完成 ✅，前端框架已就绪，等待 Rust 后端开发

**最后更新**: 2026-08-24
