# FRPC GUI 功能补齐进度

## ✅ P0 - 核心功能（已完成）

### 1. 配置管理完整字段 ✅
- [x] 传输协议选择（TCP/UDP/KCP/WebSocket）
- [x] TLS 证书文件配置（certFile, keyFile, trustedCaFile）
- [x] 心跳配置（heartbeatInterval, heartbeatTimeout）
- [x] 日志配置（logLevel, logMaxDays）
- [x] 管理控制台（adminAddr, adminPort, adminUser, adminPwd）
- [x] 高级配置（dnsServer, loginFailExit, tcpMux, udpPacketSize）

### 2. 代理管理完整字段 ✅
- [x] 带宽限制（bandwidthLimit）
- [x] 流量限制（trafficLimit）
- [x] HTTP/HTTPS 高级配置
  - [x] 自定义域名（customDomains）
  - [x] 子域名（subdomain）
  - [x] 路由（locations）
  - [x] HTTP 认证（httpUser, httpPassword）
  - [x] Host 重写（hostHeaderRewrite）
- [x] STCP/XTCP 配置
  - [x] 密钥（secretKey）
  - [x] 允许访问者（allowUsers）
- [x] 健康检查（healthCheckType, healthCheckIntervalS, healthCheckTimeoutS, healthCheckMaxUnhealthyTimes, healthCheckPath）
- [x] 插件支持预留（plugin, pluginParams）

### 3. 本地端口选择器 ✅
- [x] 调用 get_local_ports 获取监听端口
- [x] 端口选择对话框
- [x] 快速填充到代理配置

---

## 🔄 P1 - 增强功能（进行中）

### 4. 配置导入导出 🔄
- [ ] Base64 配置分享（复制/粘贴）
- [ ] 导出格式选择（TOML/JSON）
- [ ] 导入格式自动识别

### 5. 日志增强 🔄
- [ ] 日志级别过滤（DEBUG/INFO/WARN/ERROR）
- [ ] 日志搜索/过滤
- [ ] 实时日志滚动开关

### 6. 版本管理增强 🔄
- [x] 多镜像源切换（已有 get_mirrors）
- [ ] 版本详细信息展示
- [x] 下载进度显示（已有）

### 7. 首页增强 🔄
- [ ] 运行时长统计
- [ ] 连接状态显示
- [ ] 流量统计（如果有）

---

## ⏳ P2 - 锦上添花（待开始）

### 8. 国际化 ⏳
- [ ] i18n 架构
- [ ] 英文翻译
- [ ] 语言切换

### 9. UI 增强 ⏳
- [ ] 代理列表卡片/列表视图切换
- [ ] 代理搜索/过滤
- [ ] 随机代理名称生成
- [ ] 复制映射地址

### 10. 其他 ⏳
- [ ] 访客模式（Visitors）
- [ ] 多配置管理
- [ ] 配置模板

---

## 开发顺序

1. ✅ 配置管理完整字段 (P0-1) - DONE
2. ✅ 代理管理完整字段 (P0-2) - DONE
3. ✅ 本地端口选择器 (P0-3) - DONE
4. 🔄 配置导入导出 (P1-4) - IN PROGRESS
5. 🔄 日志增强 (P1-5) - IN PROGRESS
6. 🔄 首页运行时长 (P1-7) - IN PROGRESS
7. ⏳ UI 增强 (P2-9) - TODO
8. ⏳ 国际化 (P2-8) - TODO
