# 多 FRP 进程支持 - 设计方案

**开始时间**: 2026-08-25 15:26  
**优先级**: ⭐⭐⭐⭐⭐  
**预计工作量**: 50 小时

---

## 🎯 目标

支持同时运行多个独立的 FRP 进程，每个进程连接不同的 FRP 服务器。

---

## 📐 架构设计

### 当前架构（单进程）

```
AppState
├── process_manager: Option<FrpProcessManager>  ← 单个
├── config_manager: Option<ConfigManager>       ← 单个
└── ...
```

**问题**:
- 只能运行一个 FRP 进程
- 所有代理共享一个配置
- 无法连接多个服务器

---

### 新架构（多进程）

```
AppState
├── process_managers: HashMap<String, FrpProcessManager>  ← 多个
├── config_managers: HashMap<String, ConfigManager>       ← 多个
├── settings_manager: Option<SettingsManager>
├── version_manager: Option<FrpVersionManager>
└── log_tx: Option<Sender<String>>
```

**优势**:
- ✅ 每个服务器独立进程
- ✅ 独立配置文件和日志
- ✅ 独立控制（启动/停止/重启）
- ✅ 互不干扰

---

## 📁 目录结构

### 当前
```
%APPDATA%/frpc-gui/
├── settings.json
├── frpc-gui-data.json
├── frpc.toml          ← 单一配置文件
└── frpc.log           ← 单一日志文件
```

### 新结构
```
%APPDATA%/frpc-gui/
├── settings.json
├── frpc-gui-data.json
└── servers/
    ├── server-{id}/
    │   ├── config.toml    ← 独立配置
    │   └── frpc.log       ← 独立日志
    ├── server-{id}/
    │   ├── config.toml
    │   └── frpc.log
    └── ...
```

---

## 🔧 后端改动

### 1. AppState 结构调整

**文件**: `src-tauri/src/commands/mod.rs`

```rust
pub struct AppState {
    // 多个进程管理器，key 为服务器 ID
    pub process_managers: Mutex<HashMap<String, FrpProcessManager>>,
    // 多个配置管理器，key 为服务器 ID
    pub config_managers: Mutex<HashMap<String, ConfigManager>>,
    pub log_tx: Mutex<Option<mpsc::Sender<String>>>,
    pub settings_manager: Mutex<Option<SettingsManager>>,
    pub version_manager: Mutex<Option<FrpVersionManager>>,
}
```

---

### 2. 新增 API 命令

**文件**: `src-tauri/src/commands/mod.rs`

#### 启动指定服务器
```rust
#[tauri::command]
pub async fn start_server(
    server_id: String,
    state: State<'_, AppState>
) -> Result<bool, String>
```

#### 停止指定服务器
```rust
#[tauri::command]
pub async fn stop_server(
    server_id: String,
    state: State<'_, AppState>
) -> Result<bool, String>
```

#### 重启指定服务器
```rust
#[tauri::command]
pub async fn restart_server(
    server_id: String,
    state: State<'_, AppState>
) -> Result<bool, String>
```

#### 获取服务器状态
```rust
#[tauri::command]
pub async fn get_server_status(
    server_id: String,
    state: State<'_, AppState>
) -> Result<ServerStatusResponse, String>

pub struct ServerStatusResponse {
    pub running: bool,
    pub pid: Option<u32>,
    pub state: String,
    pub proxy_count: usize,  // 关联的代理数量
    pub error: Option<String>,
}
```

#### 获取所有服务器状态
```rust
#[tauri::command]
pub async fn get_all_servers_status(
    state: State<'_, AppState>
) -> Result<Vec<ServerStatusResponse>, String>
```

---

### 3. 配置管理器改动

**文件**: `src-tauri/src/frp/config.rs`

每个服务器有独立的 ConfigManager：

```rust
impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Self;
    
    // 配置文件路径：servers/server-{id}/config.toml
    pub fn get_server_config_path(server_id: &str) -> PathBuf;
}
```

---

### 4. 进程管理器改动

**文件**: `src-tauri/src/frp/process.rs`

FrpProcessManager 需要支持：
- 独立的日志输出通道
- 独立的配置文件路径
- 进程状态追踪

---

## 💻 前端改动

### 1. 服务器卡片新增控制按钮

**文件**: `src/views/Servers.vue`

```vue
<template #actions>
  <!-- 根据状态显示不同按钮 -->
  <a @click="startServer(server)" v-if="!server.running">
    <PlayCircleOutlined /> 启动
  </a>
  <a @click="stopServer(server)" v-else-if="server.running">
    <PauseCircleOutlined /> 停止
  </a>
  <a @click="restartServer(server)" v-if="server.running">
    <ReloadOutlined /> 重启
  </a>
  <a-divider type="vertical" />
  <a @click="openEdit(server)">编辑</a>
  <a @click="deleteServer(server)" class="text-danger">删除</a>
</template>

<!-- 状态显示 -->
<div class="server-status">
  <a-tag :color="statusColor">
    {{ statusText }}
  </a-tag>
  <span v-if="server.pid" class="pid-badge">PID: {{ server.pid }}</span>
  <span v-if="server.proxy_count" class="proxy-count">
    {{ server.proxy_count }} 个代理
  </span>
</div>
```

---

### 2. Store 新增状态和方法

**文件**: `src/stores/app.ts`

```typescript
// 新增状态
const serverStatuses = ref<Map<string, ServerStatus>>(new Map());

// 新增方法
async function startServer(serverId: string) {
  const server = servers.value.find(s => s.id === serverId);
  if (!server) throw new Error('服务器不存在');
  
  // 获取该服务器的所有代理
  const serverProxies = proxies.value.filter(p => p.server_id === serverId);
  
  // 构建配置
  const config: FrpConfig = {
    serverAddr: server.serverAddr,
    serverPort: server.serverPort,
    token: server.token,
    tlsEnable: server.tlsEnable,
    proxies: serverProxies,
    // ... 其他配置
  };
  
  await invoke('start_server', { serverId, config });
  await refreshServerStatus(serverId);
}

async function stopServer(serverId: string) {
  await invoke('stop_server', { serverId });
  await refreshServerStatus(serverId);
}

async function restartServer(serverId: string) {
  const server = servers.value.find(s => s.id === serverId);
  if (!server) throw new Error('服务器不存在');
  
  const serverProxies = proxies.value.filter(p => p.server_id === serverId);
  
  const config: FrpConfig = {
    serverAddr: server.serverAddr,
    serverPort: server.serverPort,
    token: server.token,
    tlsEnable: server.tlsEnable,
    proxies: serverProxies,
  };
  
  await invoke('restart_server', { serverId, config });
  await refreshServerStatus(serverId);
}

async function refreshServerStatus(serverId?: string) {
  if (serverId) {
    const status = await invoke('get_server_status', { serverId });
    serverStatuses.value.set(serverId, status);
  } else {
    const statuses = await invoke('get_all_servers_status');
    serverStatuses.value = new Map(statuses.map(s => [s.server_id, s]));
  }
}
```

---

### 3. 服务器状态轮询

```typescript
// 每 5 秒刷新一次所有服务器状态
let statusTimer: ReturnType<typeof setInterval> | null = null;

function init() {
  // ... 其他初始化
  
  if (statusTimer === null) {
    statusTimer = setInterval(() => {
      refreshServerStatus(); // 刷新所有服务器状态
    }, 5000);
  }
}
```

---

## 🔄 工作流程

### 启动服务器流程

```
1. 用户点击"启动"按钮
   ↓
2. 前端：获取该服务器的所有代理
   ↓
3. 前端：构建 FrpConfig（包含服务器配置 + 代理配置）
   ↓
4. 前端：调用 start_server(serverId, config)
   ↓
5. 后端：创建独立的配置目录 servers/server-{id}/
   ↓
6. 后端：生成 config.toml 文件
   ↓
7. 后端：启动 FrpProcessManager
   ↓
8. 后端：启动 frpc 进程
   ↓
9. 后端：返回成功
   ↓
10. 前端：刷新服务器状态
   ↓
11. 前端：显示"运行中"状态和 PID
```

---

## ⚠️ 注意事项

### 1. 向后兼容
- 保留旧的单进程 API（start_frp, stop_frp）
- 新代码使用新的多进程 API
- 迁移时自动创建新的目录结构

### 2. 资源管理
- 每个进程独立管理
- 删除服务器时自动停止并清理进程
- 应用退出时停止所有进程

### 3. 错误处理
- 某个服务器启动失败不影响其他服务器
- 进程崩溃自动重启（可选）
- 详细的错误日志

---

## 📋 开发步骤

### Step 1: 后端架构改造 (12 小时)
- [ ] 修改 AppState 结构
- [ ] 实现 HashMap 管理多个进程
- [ ] 创建独立的配置目录
- [ ] 实现进程清理机制

### Step 2: 新增 API 命令 (8 小时)
- [ ] start_server
- [ ] stop_server
- [ ] restart_server
- [ ] get_server_status
- [ ] get_all_servers_status

### Step 3: 前端状态管理 (6 小时)
- [ ] 新增 serverStatuses 状态
- [ ] 实现 startServer/stopServer/restartServer
- [ ] 实现状态轮询

### Step 4: 前端 UI 改造 (10 小时)
- [ ] 服务器卡片新增控制按钮
- [ ] 显示运行状态和 PID
- [ ] 显示代理数量
- [ ] 状态图标和颜色

### Step 5: 测试和优化 (14 小时)
- [ ] 单元测试
- [ ] 集成测试
- [ ] 性能优化
- [ ] 错误处理

---

## 🎯 成功标准

- [ ] 可以同时运行 3+ 个 FRP 进程
- [ ] 每个进程独立控制
- [ ] 进程状态实时更新
- [ ] 删除服务器自动清理进程
- [ ] 应用重启后恢复进程状态
- [ ] 无内存泄漏
- [ ] CPU 占用合理

---

## 📊 进度追踪

**开始时间**: 2026-08-25 15:26  
**预计完成**: 2026-08-28 17:00

- [ ] Step 1: 后端架构改造
- [ ] Step 2: 新增 API 命令
- [ ] Step 3: 前端状态管理
- [ ] Step 4: 前端 UI 改造
- [ ] Step 5: 测试和优化

---

**设计师**: AI Assistant  
**状态**: 设计完成，准备开发
