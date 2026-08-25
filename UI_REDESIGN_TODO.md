# UI 改造 TODO

## 任务

### 1. 服务器列表改为卡片形式
- 文件：`src/views/Servers.vue`
- 使用 `a-card` 组件替代 `a-table`
- 卡片显示：服务器名称、地址、端口、TLS 状态、启用状态
- 操作按钮：编辑、删除、设为活动

### 2. 代理列表改为卡片形式
- 文件：`src/views/Proxies.vue`
- 使用 `a-card` 组件替代 `a-table`
- 卡片显示：代理名称、类型、本地地址、远程端口、状态
- 添加"所属服务器"选择器

### 3. 代理关联服务器
- 在 ProxyConfig 中添加 `serverId` 字段
- 添加代理时选择所属服务器
- 代理卡片上显示所属服务器标签

### 4. 活动服务器切换
- 服务器卡片上添加"设为活动"按钮
- 代理页面显示当前活动服务器
- 保存配置时只保存活动服务器的配置

## 实施步骤

1. 修改 `src/types/index.ts` - 添加 `serverId` 到 `ProxyConfig`
2. 修改 `src/views/Servers.vue` - 卡片布局
3. 修改 `src/views/Proxies.vue` - 卡片布局 + 服务器选择
4. 修改 `src/stores/app.ts` - 添加活动服务器管理逻辑
