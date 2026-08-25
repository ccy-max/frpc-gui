# 服务器和代理关联设计

## 当前问题

目前的代码中：
- `Servers.vue` - 管理多个 FRP 服务器配置
- `Proxies.vue` - 管理代理配置
- **问题**：代理没有关联到具体服务器

## 解决方案

### 方案 A：单服务器 + 多代理（frpc 原生模式）

**结构**：
```
FrpConfig (主配置)
├── server_addr: "xxx"
├── server_port: 7000
├── auth: { token: "xxx" }
└── proxies: [Proxy1, Proxy2, Proxy3...]
```

**特点**：
- 一个 frpc 实例只连接一个服务器
- 所有代理都属于这个服务器
- 符合 frpc 原生设计

**UI 调整**：
- 服务器管理：只保留**一个活动服务器配置**（不是列表）
- 代理管理：显示所有代理，都属于当前活动服务器

---

### 方案 B：多服务器配置（配置文件分组）

**结构**：
```
Server1 (frpc 实例 1)
├── server_addr: "srv1.example.com"
├── server_port: 7000
└── proxies: [Proxy1, Proxy2]

Server2 (frpc 实例 2)
├── server_addr: "srv2.example.com"
├── server_port: 7001
└── proxies: [Proxy3, Proxy4, Proxy5]
```

**特点**：
- 支持多个 frpc 实例（不同端口运行）
- 每个服务器有自己的代理组
- 需要启动多个 frpc 进程

**UI 调整**：
- 服务器列表：每个服务器是一个 frpc 实例
- 代理列表：需要选择属于哪个服务器
- 可以分别启动/停止每个服务器的 frpc 进程

---

### 方案 C：服务器列表 + 代理模板（推荐）

**结构**：
```
服务器列表（连接配置）
├── Server1: srv1.example.com:7000
├── Server2: srv2.example.com:7001
└── Server3: srv3.example.com:7000

代理模板库（可复用）
├── ProxyTemplate1: web-service (TCP 8080→8080)
├── ProxyTemplate2: game-server (TCP 25565→25565)
└── ProxyTemplate3: http-service (HTTP 80→80)

部署配置（服务器 + 代理组合）
├── Deploy1: Server1 + [ProxyTemplate1, ProxyTemplate2]
└── Deploy2: Server2 + [ProxyTemplate1, ProxyTemplate3]
```

**特点**：
- 服务器和代理解耦
- 代理可以作为模板复用到不同服务器
- 灵活组合

---

## 推荐方案

**建议采用方案 A（单服务器模式）**，原因：

1. ✅ 符合 frpc 原生设计
2. ✅ 实现简单
3. ✅ 用户容易理解
4. ✅ 一个 frpc 进程管理所有代理

**如果用户需要多服务器**：
- 可以运行多个 frpc 实例（不同配置文件）
- 未来可以扩展为方案 B

---

## UI 改造建议（方案 A）

### 服务器管理页面
- 改为**卡片形式**展示
- 只有一个**活动服务器配置**
- 可以保存多个服务器配置（快速切换）

### 代理管理页面
- 改为**卡片形式**展示
- 每个代理显示所属服务器（当前活动的）
- 添加"选择服务器"下拉框（切换活动服务器）

---

## 数据结构调整

### 当前结构
```typescript
// Servers.vue
servers: [
  { id: '1', name: 'Server1', serverAddr: '...', serverPort: 7000, ... }
]

// Proxies.vue
proxies: [
  { name: 'proxy1', type: 'tcp', local_port: 8080, ... }
]
```

### 建议结构（方案 A）
```typescript
// 服务器配置（支持保存多个，但只有一个活动）
servers: [
  { id: '1', name: 'Home-Server', serverAddr: '...', serverPort: 7000, active: true },
  { id: '2', name: 'Work-Server', serverAddr: '...', serverPort: 7001, active: false }
]

// 代理配置（属于当前活动服务器）
proxies: [
  { name: 'web', type: 'tcp', local_port: 8080, remote_port: 8080, serverId: '1' }
]
```

### 建议结构（方案 B）
```typescript
// 服务器配置
servers: [
  { 
    id: '1', 
    name: 'Home-Server', 
    serverAddr: '...', 
    serverPort: 7000,
    proxies: [  // 直接嵌套代理
      { name: 'web', type: 'tcp', local_port: 8080, ... }
    ]
  }
]
```
