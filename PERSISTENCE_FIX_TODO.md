# 数据持久化修复 - 已完成 ✅

## 问题
1. ~~代理重复：addProxy 同时添加到 proxies.value 和 frpConfig.value.proxies~~ ✅ 已修复
2. ~~数据不持久化：servers 和 proxies 只存在内存中，重启后丢失~~ ✅ 已修复

## 已完成的修改

### 1. 修复代理重复问题
修改 `src/stores/app.ts` 中的 `addProxy` 函数，只添加到 `proxies.value`

### 2. 添加数据持久化
- **后端** (`src-tauri/src/commands/mod.rs`):
  - 添加 `PersistentData` 数据结构
  - 添加 `load_persistent_data()` 命令
  - 添加 `save_persistent_data()` 命令
  - 数据保存到 `frpc-gui-data.json`

- **前端** (`src/stores/app.ts`):
  - 修改 `addServer`/`updateServer`/`deleteServer` 为 async，调用 `savePersistentData()`
  - 修改 `addProxy`/`updateProxy`/`deleteProxy` 为 async，调用 `savePersistentData()`
  - 添加 `savePersistentData()` 函数
  - 添加 `loadPersistentData()` 函数
  - 在 `init()` 中调用 `loadPersistentData()`
  - 在 `resetAllConfig()` 中调用 `savePersistentData()` 清空数据

### 3. 注册后端命令
在 `src-tauri/src/lib.rs` 中注册 `load_persistent_data` 和 `save_persistent_data`

## 构建状态
- ✅ 前端构建成功 (vite build)
- ⏳ 后端构建需要 Rust 环境

## 测试步骤
1. 在 Windows 上编译运行应用
2. 新建服务器配置
3. 新建代理配置
4. 关闭应用
5. 重新打开应用
6. 验证服务器和代理数据是否保留

## 数据文件位置
Windows: `%APPDATA%\frpc-gui\frpc-gui-data.json`
