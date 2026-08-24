# FRPC GUI 开发指南

## 项目架构说明

### 前端架构

```
Vue 3 (Composition API)
    ├── Pinia (状态管理)
    ├── Vue Router (路由)
    ├── Element Plus (UI 组件)
    ├── Vue I18n (国际化)
    └── Vite (构建工具)
```

### 后端架构

```
Tauri v2
    ├── Rust 命令处理
    ├── FRP 进程管理
    ├── 配置文件解析
    └── 系统交互
```

### 通信机制

前端通过 Tauri 的 `invoke` API 调用 Rust 命令：

```typescript
// 前端调用
import { invoke } from '@tauri-apps/api/core'

const result = await invoke('load_config')
```

```rust
// Rust 命令
#[tauri::command]
fn load_config() -> Result<AppConfig, String> {
    // 处理逻辑
}
```

## 核心模块开发指南

### 1. FRP 二进制管理

**位置**: `src-tauri/src/frp/manager.rs`

**功能**:
- 检测已安装的 FRP 版本
- 从官方/镜像源下载 FRP
- 解压和安装 FRP 二进制
- 版本切换和删除

**开发要点**:
```rust
pub struct FrpManager {
    install_path: PathBuf,
    current_version: Option<String>,
}

impl FrpManager {
    pub async fn download_version(&self, version: &str) -> Result<()>;
    pub async fn list_versions(&self) -> Vec<FrpVersion>;
    pub fn get_binary_path(&self, version: &str) -> PathBuf;
}
```

### 2. FRP 配置管理

**位置**: `src-tauri/src/frp/config.rs`

**功能**:
- 解析 frpc.toml / frpc.ini
- 生成配置文件
- 验证配置有效性
- 配置导入导出

**配置结构**:
```rust
pub struct FrpConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub auth: AuthConfig,
    pub proxies: Vec<ProxyConfig>,
}
```

### 3. FRP 进程管理

**位置**: `src-tauri/src/frp/process.rs`

**功能**:
- 启动 FRP 进程
- 停止 FRP 进程
- 重启 FRP 进程
- 监控进程状态
- 捕获进程输出（日志）

**开发要点**:
```rust
pub struct FrpProcess {
    child: Option<Child>,
    server_id: String,
    log_tx: mpsc::Sender<LogEntry>,
}

impl FrpProcess {
    pub async fn start(&mut self, config: &FrpConfig) -> Result<()>;
    pub async fn stop(&mut self) -> Result<()>;
    pub fn is_running(&self) -> bool;
}
```

### 4. 日志系统

**位置**: `src-tauri/src/logger.rs`

**功能**:
- 应用日志记录
- FRP 日志捕获
- 日志文件轮转
- 日志查询和导出

**实现方式**:
- 使用 `tracing`  crate 进行结构化日志
- 通过管道捕获 FRP 进程的 stdout/stderr
- 前端通过 WebSocket 或轮询获取实时日志

### 5. 系统托盘

**位置**: `src-tauri/src/tray.rs`

**功能**:
- 创建系统托盘图标
- 托盘右键菜单
- 快速启动/停止代理
- 显示/隐藏主窗口

**配置**:
```rust
fn create_tray(app: &AppHandle) -> SystemTray {
    let menu = SystemTrayMenu::new()
        .add_item(MenuItem::with_id("show", "显示主窗口"))
        .add_item(MenuItem::with_id("quit", "退出"));
    
    SystemTray::new().with_menu(menu)
}
```

## 前端组件开发规范

### 1. 组件命名

- 使用 PascalCase 命名组件文件
- 视图组件放在 `views/` 目录
- 通用组件放在 `components/` 目录

### 2. 状态管理

```typescript
// stores/app.ts
import { defineStore } from 'pinia'

export const useAppStore = defineStore('app', () => {
  // State
  const servers = ref<ServerConfig[]>([])
  
  // Getters
  const activeServers = computed(() => 
    servers.value.filter(s => s.enabled)
  )
  
  // Actions
  function addServer(server: ServerConfig) {
    servers.value.push(server)
  }
  
  return { servers, activeServers, addServer }
})
```

### 3. Tauri API 调用

```typescript
// utils/frp.ts
import { invoke } from '@tauri-apps/api/core'

export async function startProxy(serverId: string) {
  try {
    await invoke('start_proxy', { serverId })
    return { success: true }
  } catch (error) {
    return { success: false, error: String(error) }
  }
}
```

### 4. 国际化

```typescript
// i18n/locales.ts
export const zhCN = {
  server: {
    title: '服务器管理',
    addServer: '添加服务器',
  }
}

// 组件中使用
const { t } = useI18n()
<h2>{{ t('server.title') }}</h2>
```

## Rust 开发规范

### 1. 错误处理

使用 `anyhow` 进行错误处理：

```rust
use anyhow::{Result, Context};

pub fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {:?}", path))?;
    
    toml::from_str(&content)
        .with_context(|| "Failed to parse TOML")
}
```

### 2. 异步编程

使用 `tokio` 异步运行时：

```rust
use tokio::sync::mpsc;

#[tauri::command]
async fn download_frp(version: String) -> Result<(), String> {
    // 异步下载逻辑
}
```

### 3. 线程安全

使用 `Arc<Mutex<T>>` 或 `Arc<RwLock<T>>` 进行状态共享：

```rust
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub processes: Arc<Mutex<HashMap<String, FrpProcess>>>,
}
```

## 调试技巧

### 前端调试

1. 在开发模式下，使用浏览器 DevTools
2. 使用 `console.log` 输出调试信息
3. 使用 Vue DevTools 检查组件状态

### 后端调试

1. 使用 `log::info!` / `log::debug!` 输出日志
2. 使用 `dbg!` 宏快速调试
3. 查看 `stdout` 和 `stderr` 输出

### Tauri 调试

```rust
// 在开发模式下打开开发者工具
#[cfg(debug_assertions)]
app.get_webview_window("main").unwrap().open_devtools();
```

## 性能优化

### 前端优化

1. 使用懒加载路由
2. 大数据列表使用虚拟滚动
3. 避免不必要的组件重新渲染
4. 使用 `shallowRef` 优化大型对象

### 后端优化

1. 使用连接池管理 FRP 进程
2. 异步 I/O 操作
3. 合理使用缓存
4. 避免阻塞主线程

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_config() {
        // 测试配置解析
    }
}
```

### 集成测试

```typescript
// 测试 Tauri 命令
describe('FRP Commands', () => {
  it('should load config', async () => {
    const result = await invoke('load_config')
    expect(result).toBeDefined()
  })
})
```

## 常见问题

### Q: 如何处理 FRP 版本兼容性？

A: 维护一个版本兼容性矩阵，在下载前检查：

```rust
const COMPATIBLE_VERSIONS = ["0.52.0", "0.53.0", "0.54.0"];
```

### Q: 如何实现跨平台路径处理？

A: 使用 `std::path::PathBuf` 和 `dirs` crate：

```rust
let config_dir = dirs::config_dir()
    .unwrap()
    .join("frpc-gui");
```

### Q: 如何处理权限问题？

A: 在需要时请求系统权限，如开机自启：

```rust
// 使用 tauri-plugin-autostart
tauri_plugin_autostart::init(...);
```

## 下一步开发

1. 完善 Rust 后端的 FRP 管理逻辑
2. 实现实时日志流
3. 添加系统托盘功能
4. 实现自动更新
5. 完善单元测试
6. 编写用户文档

---

**提示**: 开发过程中遇到问题，优先查阅：
- [Tauri v2 文档](https://v2.tauri.app/)
- [Vue 3 文档](https://vuejs.org/)
- [Element Plus 文档](https://element-plus.org/)
- [Rust 文档](https://doc.rust-lang.org/)
