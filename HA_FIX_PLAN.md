# 高可用全面修复计划 — 全部完成 ✅

## Phase 1: 严重问题 🔴
- [x] S1: 恢复 load_config/save_config/import_toml_config（default 目录薄包装 + spawn_blocking）
- [x] S2: Admin API 凭据动态化（FrpProcessManager.admin_endpoint 按服务器快照）

## Phase 2: 中等问题 🟡
- [x] M1: settings.json 原子写入 + version 字段迁移 (v0→v1)
- [x] M2: process.rs 锁中毒自愈（12处 into_inner）
- [x] M3: mock-tauri 仅 DEV+非Tauri 动态加载，生产 Tree-Shaking 剔除
- [x] M4: refreshProxyStatus 状态变化检测 → log_connection_event 持久化
- [x] M5: App.vue onUnmounted 调用 cleanup()

## Phase 3: 轻微问题 🟢
- [x] L2: open_url scheme 白名单 (http/https/mailto)
- [x] L4: vite manualChunks 分包（业务 1619KB→97KB）
- [x] L7: utils/mod.rs #![allow(dead_code)] 消噪

## Phase 4: 验证结果
- cargo check: 零 error（警告 28→18，全部可接受类）
- cargo build: 完整链接成功 30.21s
- npm run build: 成功 8.10s，分包生效

## 引入的高可用机制
1. **原子持久化**: settings.json 与 monitoring-data.json 统一 temp+rename
2. **数据版本迁移**: AppSettings.version v0→v1 自动补齐
3. **锁中毒自愈**: 后台线程 Mutex panic 不再连锁崩溃
4. **优雅降级**: 连接事件记录失败仅告警不阻断监控主流程
5. **输入校验**: URL scheme 白名单防协议滥用
6. **环境隔离**: Mock 三重守卫(DEV+非Tauri+动态import)
7. **资源生命周期**: 全局轮询定时器随组件卸载清理
8. **缓存优化**: vendor 分包使业务迭代不失效第三方长缓存
