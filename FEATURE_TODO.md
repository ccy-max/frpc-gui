# FRPC GUI 功能补齐计划

## P0 - 核心功能（必须）

### 1. 配置管理完整字段
- [ ] 传输协议选择（TCP/UDP/KCP/WebSocket）
- [ ] TLS 证书文件配置（certFile, keyFile, trustedCaFile）
- [ ] 心跳配置（heartbeatInterval, heartbeatTimeout）
- [ ] 日志配置（logLevel, logMaxDays）
- [ ] 管理控制台（adminAddr, adminPort, adminUser, adminPwd）
- [ ] 高级配置（dnsServer, loginFailExit, tcpMux, udpPacketSize）

### 2. 代理管理完整字段
- [ ] 带宽限制（bandwidthLimit）
- [ ] 流量限制（trafficLimit）
- [ ] HTTP/HTTPS 高级配置
  - [ ] 自定义域名（customDomains）
  - [ ] 子域名（subdomain）
  - [ ] 路由（locations）
  - [ ] HTTP 认证（httpUser, httpPassword）
- [ ] STCP/XTCP 配置
  - [ ] 密钥（secretKey）
  - [ ] 允许访问者（allowUsers）
- [ ] 健康检查（healthCheckType, healthCheckIntervalS, healthCheckTimeoutS, healthCheckMaxUnhealthyTimes, healthCheckPath）
- [ ] 插件支持（plugin, pluginParams）

### 3. 本地端口选择器
- [ ] 调用 get_local_ports 获取监听端口
- [ ] 端口选择对话框
- [ ] 快速填充到代理配置

## P1 - 增强功能（重要）

### 4. 配置导入导出
- [ ] Base64 配置分享（复制/粘贴）
- [ ] 导出格式选择（TOML/JSON）
- [ ] 导入格式自动识别

### 5. 日志增强
- [ ] 日志级别过滤（DEBUG/INFO/WARN/ERROR）
- [ ] 日志搜索/过滤
- [ ] 实时日志滚动开关

### 6. 版本管理增强
- [ ] 多镜像源切换
- [ ] 版本详细信息展示
- [ ] 下载进度显示（已有）

### 7. 首页增强
- [ ] 运行时长统计
- [ ] 连接状态显示
- [ ] 流量统计（如果有）

## P2 - 锦上添花

### 8. 国际化
- [ ] i18n 架构
- [ ] 英文翻译
- [ ] 语言切换

### 9. UI 增强
- [ ] 代理列表卡片/列表视图切换
- [ ] 代理搜索/过滤
- [ ] 随机代理名称生成
- [ ] 复制映射地址

### 10. 其他
- [ ] 访客模式（Visitors）
- [ ] 多配置管理
- [ ] 配置模板

---

## 开发顺序

1. 配置管理完整字段（P0-1）
2. 代理管理完整字段（P0-2）
3. 本地端口选择器（P0-3）
4. 配置导入导出（P1-4）
5. 日志增强（P1-5）
6. 首页运行时长（P1-7）
7. UI 增强（P2-9）
8. 国际化（P2-8）
