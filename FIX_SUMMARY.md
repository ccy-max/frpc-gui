# 修复总结

## 问题描述
用户报告了两个问题：
1. **代理重复问题**：新建一个代理会多出一个重复的
2. **数据持久化问题**：关闭窗口后，之前新建的服务和代理都消失了

## 根本原因分析

### 1. 代理重复问题
在 `src/stores/app.ts` 的 `addProxy` 函数中：
```typescript
function addProxy(data: any) {
  proxies.value.push(data);  // 添加到 UI 列表
  if (frpConfig.value) frpConfig.value.proxies.push(data);  // 又添加到配置
}
```
数据被同时添加到两个地方，导致重复显示。

### 2. 数据持久化问题
- `servers` 和 `proxies` 数组只存在于前端 Pinia store 的内存中
- 没有后端 API 来保存这些数据到磁盘
- 应用重启后，内存数据丢失

## 修复方案

### 修复 1：代理重复
**文件**: `src/stores/app.ts`

**修改前**:
```typescript
function addProxy(data: any) {
  proxies.value.push(data);
  if (frpConfig.value) frpConfig.value.proxies.push(data);
}
```

**修改后**:
```typescript
async function addProxy(data: any) {
  // 只添加到 proxies.value，避免重复
  proxies.value.push(data);
  await savePersistentData();
}
```

同时对 `updateProxy` 和 `deleteProxy` 也做了相同的简化。

### 修复 2：数据持久化

#### 后端修改 (`src-tauri/src/commands/mod.rs`)

1. **添加数据结构**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentData {
    pub servers: Vec<serde_json::Value>,
    pub proxies: Vec<serde_json::Value>,
}
```

2. **添加加载命令**:
```rust
#[tauri::command]
pub fn load_persistent_data() -> Result<PersistentData, String> {
    let path = get_persistent_data_path()?;
    if !path.exists() {
        return Ok(PersistentData::default());
    }
    // 读取并解析 JSON 文件
}
```

3. **添加保存命令**:
```rust
#[tauri::command]
pub fn save_persistent_data(data: PersistentData) -> Result<bool, String> {
    let path = get_persistent_data_path()?;
    let content = serde_json::to_string_pretty(&data)?;
    std::fs::write(&path, content)?;
    Ok(true)
}
```

#### 前端修改 (`src/stores/app.ts`)

1. **服务器管理改为异步并保存**:
```typescript
async function addServer(data: any) {
  servers.value.push(data);
  await savePersistentData();
}
```

2. **代理管理改为异步并保存**:
```typescript
async function addProxy(data: any) {
  proxies.value.push(data);
  await savePersistentData();
}
```

3. **添加持久化函数**:
```typescript
async function savePersistentData() {
  await invoke<boolean>('save_persistent_data', {
    data: { servers: servers.value, proxies: proxies.value }
  });
}

async function loadPersistentData() {
  const data = await invoke<any>('load_persistent_data');
  servers.value = data.servers || [];
  proxies.value = data.proxies || [];
}
```

4. **初始化时加载**:
```typescript
function init() {
  // ... 其他初始化
  loadPersistentData(); // 加载持久化数据
}
```

#### 注册命令 (`src-tauri/src/lib.rs`)

在 `invoke_handler` 中添加：
```rust
commands::load_persistent_data,
commands::save_persistent_data,
```

## 修改的文件列表

1. `frpc-gui/src/stores/app.ts` - 修复代理重复 + 添加持久化逻辑
2. `frpc-gui/src-tauri/src/commands/mod.rs` - 添加后端持久化命令
3. `frpc-gui/src-tauri/src/lib.rs` - 注册新命令

## 数据文件位置

- **Windows**: `%APPDATA%\frpc-gui\frpc-gui-data.json`
- **macOS**: `~/Library/Application Support/frpc-gui/frpc-gui-data.json`
- **Linux**: `~/.config/frpc-gui/frpc-gui-data.json`

## 验证步骤

1. ✅ 前端构建成功 (`npm run build`)
2. ⏳ 后端构建需要 Rust 环境（当前环境未安装）
3. ⏳ 功能测试需要在 Windows 上运行完整应用

## 预期效果

修复后：
- ✅ 新建代理不会重复
- ✅ 新建服务器和代理会自动保存到磁盘
- ✅ 关闭应用后重新打开，数据依然存在
- ✅ 重置配置时会自动清空持久化数据
