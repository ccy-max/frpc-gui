import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import type { FrpConfig, ProcessStatus, LogEntry } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';

export const useAppStore = defineStore('app', () => {
  // 应用设置（持久化）
  const theme = ref<'light' | 'dark' | 'auto'>('auto');
  const language = ref<'zh-CN' | 'en-US'>('zh-CN');
  const autoStart = ref(false);
  const minimizeToTray = ref(true);
  const closeToTray = ref(true);
  const defaultServerId = ref<string | null>(null);  // 默认服务器 ID

  // FRP 配置
  const frpConfig = ref<FrpConfig | null>(null);

  // 进程状态
  const processStatus = ref<ProcessStatus>({
    running: false,
    pid: null,
    state: 'stopped',
  });

  // 日志
  const logs = ref<LogEntry[]>([]);
  const frpcLogContent = ref('');
  const autoScrollLogs = ref(true);

  // 服务器和代理列表
  const servers = ref<any[]>([]);
  const proxies = ref<any[]>([]);
  const versions = ref<any[]>([]);
  const downloadedVersions = ref<any[]>([]);
  const localPorts = ref<any[]>([]);
  const mirrors = ref<any[]>([]);
  
  // 服务器状态（多进程支持）
  const serverStatuses = ref<Map<string, any>>(new Map());
  const proxyStatuses = ref<Map<string, any>>(new Map());  // 代理状态
  const serverTraffic = ref<Map<string, any>>(new Map());  // 服务器流量
  // FRP 启动时刻（全局，修复：运行时长曾存组件本地，切页即重置）
  // 从 localStorage 恢复初始值（右键刷新后兜底，轮询会用后端 last_start_time 覆盖）
  const frpcStartedAt = ref<number | null>(
    Number(localStorage.getItem('frpcStartedAt')) || null
  );
  // 持久化到 localStorage（刷新兜底，防止运行时长归零）
  watch(frpcStartedAt, (v) => {
    if (v) localStorage.setItem('frpcStartedAt', String(v));
    else localStorage.removeItem('frpcStartedAt');
  });
  // 实时日志缓冲（全局，修复：曾存组件本地，切页即清空）
  const liveLogs = ref<{ ts: string; line: string; isError: boolean }[]>([]);
  const MAX_LIVE_LOGS = 500;

  /// 追加实时日志（后端 frpc-log 事件驱动）
  function appendLiveLog(line: string, timestamp: number) {
    const d = new Date(timestamp);
    const ts = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`;
    liveLogs.value.push({ ts, line, isError: /ERR|error|失败|failed/i.test(line) });
    if (liveLogs.value.length > MAX_LIVE_LOGS) {
      liveLogs.value.splice(0, liveLogs.value.length - MAX_LIVE_LOGS);
    }
  }

  function clearLiveLogs() {
    liveLogs.value = [];
  }

  /// 从磁盘日志预填最近内容（概览挂载时实时缓冲为空则加载）
  async function loadRecentLogsFromDisk() {
    if (liveLogs.value.length > 0) return;
    try {
      const content = await invoke<string>('get_frpc_log_content');
      if (!content) return;
      const lines = content.trim().split('\n').slice(-100);
      for (const l of lines) {
        // 磁盘格式: [2026/8/26 18:50:33] [sid][FRP] xxx
        const m = l.match(/^\[(\d{4})\/(\d{1,2})\/(\d{1,2}) (\d{1,2}):(\d{2}):(\d{2})\]\s(.*)$/);
        if (m) {
          // 用日志行的真实时间戳（而非当前时间），避免历史行显示成"刚刚"
          const ts = new Date(+m[1], +m[2] - 1, +m[3], +m[4], +m[5], +m[6]).getTime();
          appendLiveLog(m[7], ts);
        } else {
          appendLiveLog(l, Date.now());
        }
      }
    } catch { /* 磁盘日志不存在时静默 */ }
  }
  const trafficHistory = ref<any[]>([]);  // 流量历史
  const connectionHistory = ref<any[]>([]);  // 连接历史

  // 计算属性
  // 多进程模式：任一服务器 running 即视为整体运行
  // 历史 bug：基于旧 processStatus（单进程），多进程下永远 false
  const isRunning = computed(() => {
    for (const s of serverStatuses.value.values()) {
      if (s?.running) return true;
    }
    return false;
  });
  const runningServersCount = computed(() => {
    let n = 0;
    for (const s of serverStatuses.value.values()) {
      if (s?.running) n++;
    }
    return n;
  });
  // 修复：活跃代理应基于实际代理列表（proxies），而非 frpConfig（延迟加载且常为 null）
  const activeProxies = computed(() => {
    return proxies.value.filter(p =>
      p.enabled !== false &&
      !p.is_range_port &&
      p.visitors_model !== 'visitors'
    );
  });
  const activeProxiesCount = computed(() => activeProxies.value.length);

  // ===== 设置持久化 =====
  async function loadSettings() {
    try {
      const s = await invoke<any>('load_settings');
      theme.value = s.theme || 'auto';
      language.value = s.language || 'zh-CN';
      autoStart.value = s.auto_start || false;
      minimizeToTray.value = s.minimize_to_tray ?? true;
      closeToTray.value = s.close_to_tray ?? true;
      defaultServerId.value = s.default_server_id || null;
      applyTheme(theme.value);
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async function saveSettings() {
    try {
      await invoke<boolean>('save_settings', {
        settings: {
          language: language.value,
          theme: theme.value,
          auto_start: autoStart.value,
          minimize_to_tray: minimizeToTray.value,
          close_to_tray: closeToTray.value,
          default_server_id: defaultServerId.value,
        }
      });
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }

  function setTheme(t: 'light' | 'dark' | 'auto') {
    theme.value = t;
    applyTheme(t);
    saveSettings();
  }

  function applyTheme(t: 'light' | 'dark' | 'auto') {
    const root = document.documentElement;
    if (t === 'auto') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.classList.toggle('dark', prefersDark);
    } else {
      root.classList.toggle('dark', t === 'dark');
    }
  }

  function setLanguage(lang: 'zh-CN' | 'en-US') {
    language.value = lang;
    saveSettings();
  }

  function setDefaultServerId(id: string | null) {
    defaultServerId.value = id;
    saveSettings();
  }


  // ===== 配置管理 =====
  async function loadConfig() {
    try {
      const result = await invoke<any>('load_config');
      if (result.success && result.config) {
        // 只更新 frpConfig，严禁覆盖 proxies！
        // proxies 的唯一可信来源是持久化数据（loadPersistentData），
        // 此处覆盖会导致启动竞态：持久化先加载显示(闪一下) →
        // 本函数后完成用 default 配置的空 proxies 覆盖(没了)
        frpConfig.value = result.config;
      }
      return result;
    } catch (e) {
      return { success: false, config: null, error: String(e) };
    }
  }

  async function saveConfig(config: FrpConfig) {
    try {
      // 字段类型对齐后端 FrpConfig：
      // - server_port 是 u16（数字），不能转字符串（历史 bug：String 会被 serde 拒绝）
      // - proxies 的 local_port/remote_port 是 Option<String>，统一字符串化
      const configToSave = {
        ...config,
        server_port: Number(config.server_port) || 0,
        proxies: config.proxies?.map(p => ({
          ...p,
          local_port: p.local_port != null ? String(p.local_port) : null,
          remote_port: p.remote_port != null ? String(p.remote_port) : null,
        })) || [],
      };
      const result = await invoke<any>('save_config', { config: configToSave });
      if (result.success && result.config) {
        frpConfig.value = result.config;
      }
      return result;
    } catch (e) {
      return { success: false, config: null, error: String(e) };
    }
  }

  async function resetAllConfig() {
    try {
      await invoke<boolean>('reset_all_config');
      frpConfig.value = null;
      proxies.value = [];
      servers.value = [];
      logs.value = [];
      versions.value = [];
      frpcStartedAt.value = null;
      // 同时清空持久化数据
      await savePersistentData();
      return { success: true };
    } catch (e) {
      return { success: false, error: String(e) };
    }
  }

  async function importTomlConfig() {
    try {
      const selected = await openDialog({
        title: '选择 frpc.toml 文件',
        filters: [{ name: 'TOML 配置', extensions: ['toml'] }],
      });
      if (selected) {
        const config = await invoke<any>('import_toml_config', { tomlPath: selected as string });
        // TOML 文件无 enabled 字段（serde 默认 false），若不修正
        // 导入的代理会被 generate_toml 的 enabled 过滤器全部跳过——
        // 表现为"导入成功但代理永不生效"
        config.proxies = (config.proxies || []).map((p: any) => ({ ...p, enabled: true }));
        frpConfig.value = config;
        proxies.value = config.proxies;
        // 持久化到 frpc-gui-data.json，重启后不丢失
        await savePersistentData();
        return { success: true };
      }
      return { success: false, error: 'canceled' };
    } catch (e) {
      return { success: false, error: String(e) };
    }
  }

  // ===== 服务器管理 =====
  async function addServer(data: any) {
    servers.value.push(data);
    await savePersistentData();
  }
  async function updateServer(id: string, updates: any) {
    const i = servers.value.findIndex(s => s.id === id);
    if (i !== -1) {
      servers.value[i] = { ...servers.value[i], ...updates };
      await savePersistentData();
    }
  }
  async function deleteServer(id: string) {
    servers.value = servers.value.filter(s => s.id !== id);
    await savePersistentData();
  }

  // ===== 代理管理 =====
  async function addProxy(data: any) {
    // 只添加到 proxies.value，避免重复
    proxies.value.push(data);
    await savePersistentData();
  }
  async function updateProxy(name: string, updates: any) {
    const i = proxies.value.findIndex(p => p.name === name);
    if (i !== -1) {
      proxies.value[i] = { ...proxies.value[i], ...updates };
      await savePersistentData();
    }
  }
  async function deleteProxy(name: string) {
    proxies.value = proxies.value.filter(p => p.name !== name);
    await savePersistentData();
  }

  async function modifyProxyStatus(name: string, enabled: boolean) {
    try {
      await invoke<boolean>('modify_proxy_status', { proxyName: name, enabled });
      updateProxy(name, { enabled });
      return { success: true };
    } catch (e) {
      return { success: false, error: String(e) };
    }
  }

  // ===== 多服务器 FRP 进程控制 =====
  /// 构建完整 FrpConfig（前后端契约：snake_case，严格对齐后端 Rust FrpConfig）
  /// start / restart 共用，消除此前 restart 用 camelCase 导致 serde 静默丢字段的 bug
  function buildServerConfig(server: any, serverProxies: any[]) {
    return {
      // 必填：服务器地址与端口
      server_addr: String(server.serverAddr ?? ''),
      server_port: Number(server.serverPort ?? 7000),
      user: null,
      // 认证：generate_toml 从 auth.method/auth.token 读取
      auth: {
        method: server.token ? 'token' : 'none',
        token: server.token || null,
        additional: null,
      },
      tls: {
        enable: !!server.tlsEnable,
        cert_file: null,
        key_file: null,
        trusted_ca_file: null,
      },
      log: { to: 'console', level: 'info', max_days: 7 },
      admin: { addr: '127.0.0.1', port: 7400, user: 'admin', password: 'admin' },
      transport: {},
      web_server: { addr: '127.0.0.1', port: 0, user: null, password: null },
      metadatas: null,
      login_fail_exit: false,
      udp_packet_size: 1500,
      // 端口统一字符串化（后端 Option<String>），null 保 null 不转 "null"
      proxies: serverProxies.map(p => ({
        ...p,
        local_port: p.local_port != null ? String(p.local_port) : null,
        remote_port: p.remote_port != null ? String(p.remote_port) : null,
      })),
      visitors: [],
    };
  }

  async function startServer(serverId: string) {
    const server = servers.value.find(s => s.id === serverId);
    if (!server) throw new Error('服务器不存在');

    const serverProxies = proxies.value.filter(p => p.server_id === serverId);
    const config = buildServerConfig(server, serverProxies);

    await invoke('start_server', { serverId, config });
    frpcStartedAt.value = Date.now();
    await refreshServerStatus(serverId);
  }

  async function stopServer(serverId: string) {
    await invoke('stop_server', { serverId });
    frpcStartedAt.value = null;
    await refreshServerStatus(serverId);
  }

  async function restartServer(serverId: string) {
    const server = servers.value.find(s => s.id === serverId);
    if (!server) throw new Error('服务器不存在');

    const serverProxies = proxies.value.filter(p => p.server_id === serverId);
    // 复用 buildServerConfig —— 与 start 完全一致（修复 camelCase 契约断裂）
    const config = buildServerConfig(server, serverProxies);

    await invoke('restart_server', { serverId, config });
    frpcStartedAt.value = Date.now();
    await refreshServerStatus(serverId);
  }

  async function refreshServerStatus(serverId?: string) {
    try {
      if (serverId) {
        const status = await invoke<any>('get_server_status', { serverId });
        serverStatuses.value.set(serverId, status);
        // 从后端同步启动时刻（应用重启后恢复运行时长）
        if (status.running && status.last_start_time) {
          frpcStartedAt.value = status.last_start_time * 1000;
        }
      } else {
        const statuses = await invoke<any[]>('get_all_servers_status');
        // 触发响应式：用新 Map 替换（确保 computed 重新计算）
        serverStatuses.value = new Map(statuses.map(s => [s.server_id, s]));
        // 同步任一运行中服务器的启动时刻
        const running = statuses.find(s => s.running);
        if (running && running.last_start_time) {
          frpcStartedAt.value = running.last_start_time * 1000;
        } else if (!running) {
          frpcStartedAt.value = null;
        }
      }
    } catch (e) {
      console.error('Failed to refresh server status:', e);
    }
  }

  // ===== 监控功能 =====
  async function refreshProxyStatus() {
    try {
      const statuses = await invoke<any[]>('get_all_proxy_status');

      // 状态变化检测：对比新旧快照，向后端持久化连接/断开事件
      const prev = proxyStatuses.value;
      for (const s of statuses) {
        const key = `${s.server_id}-${s.name}`;
        const old = prev.get(key);
        if (old && old.state !== s.state) {
          const eventType =
            s.state === 'online' ? 'connected' :
            s.state === 'offline' ? 'disconnected' : null;
          if (eventType) {
            // 异步记录，失败不影响主流程（优雅降级：仅控制台告警）
            invoke('log_connection_event', {
              proxyName: s.name,
              serverId: s.server_id,
              eventType,
              message: s.err_msg ?? null,
              durationSecs: null,
            }).catch(err => console.warn('[monitor] 记录连接事件失败:', err));
          }
        }
      }

      proxyStatuses.value = new Map(statuses.map(s => [`${s.server_id}-${s.name}`, s]));
    } catch (e) {
      console.error('Failed to refresh proxy status:', e);
    }
  }

  async function refreshServerTraffic(serverId?: string) {
    try {
      if (serverId) {
        const traffic = await invoke('get_server_traffic', { serverId });
        serverTraffic.value.set(serverId, traffic);
      } else {
        // 刷新所有服务器的流量
        for (const server of servers.value) {
          try {
            const traffic = await invoke('get_server_traffic', { serverId: server.id });
            serverTraffic.value.set(server.id, traffic);
          } catch (e) {
            console.error(`Failed to get traffic for server ${server.id}:`, e);
          }
        }
      }
    } catch (e) {
      console.error('Failed to refresh server traffic:', e);
    }
  }

  // 获取代理的状态
  function getProxyStatus(serverId: string, proxyName: string): any {
    return proxyStatuses.value.get(`${serverId}-${proxyName}`);
  }

  // 获取服务器的流量
  function getServerTraffic(serverId: string): any {
    return serverTraffic.value.get(serverId);
  }

  /// 若指定服务器的 FRP 进程正在运行则热重启（使代理变更生效），
  /// 未运行则跳过。返回是否执行了重启。
  async function restartServerIfRunning(serverId: string): Promise<boolean> {
    const status = serverStatuses.value.get(serverId);
    if (status?.running) {
      await restartServer(serverId);
      return true;
    }
    return false;
  }

  // 格式化流量显示
  function formatTraffic(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
    return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
  }

  // ===== 历史数据加载 =====
  async function loadTrafficHistory(days: number = 30) {
    try {
      trafficHistory.value = await invoke('get_traffic_history', { days });
    } catch (e) {
      console.error('Failed to load traffic history:', e);
      trafficHistory.value = [];
    }
  }

  async function loadConnectionHistory(proxyName?: string, serverId?: string) {
    try {
      connectionHistory.value = await invoke('get_connection_history', { proxyName, serverId });
    } catch (e) {
      console.error('Failed to load connection history:', e);
      connectionHistory.value = [];
    }
  }

  // ===== FRP 进程控制 =====
  /// 防御性归一化：端口字段统一字符串化（后端 ProxyConfig 为 Option<String>），
  /// 并深拷贝避免直接修改 store 状态
  function normalizeFrpConfig(raw: any): any {
    const cfg = JSON.parse(JSON.stringify(raw));
    if (Array.isArray(cfg.proxies)) {
      cfg.proxies = cfg.proxies.map((p: any) => ({
        ...p,
        local_port: p.local_port != null ? String(p.local_port) : null,
        remote_port: p.remote_port != null ? String(p.remote_port) : null,
      }));
    }
    if (Array.isArray(cfg.visitors)) {
      cfg.visitors = cfg.visitors.map((p: any) => ({
        ...p,
        local_port: p.local_port != null ? String(p.local_port) : null,
        remote_port: p.remote_port != null ? String(p.remote_port) : null,
        bind_port: p.bind_port != null ? Number(p.bind_port) : null,
      }));
    }
    return cfg;
  }

  async function startFRP() {
    if (!frpConfig.value) return { success: false, error: '没有配置' };
    try {
      await invoke<boolean>('start_frp', { config: normalizeFrpConfig(frpConfig.value) });
      await refreshProcessStatus();
      addLog({ timestamp: Date.now(), level: 'info', message: 'FRP 进程已启动', source: 'app' });
      return { success: true };
    } catch (e) {
      addLog({ timestamp: Date.now(), level: 'error', message: `启动 FRP 失败：${String(e)}`, source: 'app' });
      return { success: false, error: String(e) };
    }
  }

  async function stopFRP() {
    try {
      await invoke<boolean>('stop_frp');
      await refreshProcessStatus();
      addLog({ timestamp: Date.now(), level: 'info', message: 'FRP 进程已停止', source: 'app' });
      return { success: true };
    } catch (e) { return { success: false, error: String(e) }; }
  }

  async function restartFRP() {
    if (!frpConfig.value) return { success: false, error: '没有配置' };
    try {
      await invoke<boolean>('restart_frp', { config: normalizeFrpConfig(frpConfig.value) });
      await refreshProcessStatus();
      return { success: true };
    } catch (e) { return { success: false, error: String(e) }; }
  }

  async function reloadFRP() {
    if (!frpConfig.value) return { success: false, error: '没有配置' };
    try {
      await invoke<boolean>('reload_frp', { config: frpConfig.value });
      addLog({ timestamp: Date.now(), level: 'info', message: '配置已热重载', source: 'app' });
      return { success: true };
    } catch (e) { return { success: false, error: String(e) }; }
  }

  async function refreshProcessStatus() {
    try {
      const status = await invoke<any>('get_process_status');
      processStatus.value = {
        running: status.running,
        pid: status.pid,
        state: status.state,
      };
      // frpcStartedAt 由 refreshServerStatus 统一管理（基于后端 last_start_time）
      // 旧逻辑基于 get_process_status（单进程模式），多进程下返回 not_running
      // 会错误清空 frpcStartedAt，导致运行时长归零
    } catch (e) { console.error('Failed to get process status:', e); }
  }

  async function detectFrpcProcess() {
    try {
      const found = await invoke<boolean>('detect_frpc_process');
      if (found) {
        await refreshProcessStatus();
        addLog({ timestamp: Date.now(), level: 'info', message: '检测到运行中的 FRP 进程', source: 'app' });
      }
      return found;
    } catch (e) { return false; }
  }

  // ===== 日志 =====
  function addLog(entry: LogEntry) {
    logs.value.push(entry);
    if (logs.value.length > 1000) logs.value = logs.value.slice(-500);
  }
  function clearLogs() { logs.value = []; }

  async function loadFrpcLogContent() {
    try {
      const content = await invoke<string>('get_frpc_log_content');
      frpcLogContent.value = content;
      return content;
    } catch (e) {
      console.error('Failed to load frpc log:', e);
      return '';
    }
  }

  async function openFrpcLogFile() {
    try {
      await invoke<boolean>('open_frpc_log_file');
    } catch (e) {
      console.error('Failed to open log file:', e);
    }
  }

  // ===== FRP 版本管理 =====
  async function loadVersions() {
    try {
      const list = await invoke<any[]>('list_frp_versions');
      versions.value = list || [];
    } catch (e) {
      console.error('Failed to load versions:', e);
      versions.value = [];
    }
  }

  async function loadDownloadedVersions() {
    try {
      const list = await invoke<any[]>('get_downloaded_versions');
      downloadedVersions.value = list || [];
    } catch (e) {
      console.error('Failed to load downloaded versions:', e);
    }
  }

  async function downloadVersion(version: string, url: string) {
    try {
      const path = await invoke<string>('download_frp_version', { version, url });
      await loadVersions();
      return { success: true, path };
    } catch (e) { return { success: false, error: String(e) }; }
  }

  async function deleteVersion(version: string) {
    try {
      await invoke<boolean>('delete_frp_version', { version });
      await loadVersions();
      return { success: true };
    } catch (e) { return { success: false, error: String(e) }; }
  }

  /// 设置当前使用的 FRP 版本（版本管理「使用此版本」）
  async function setActiveVersion(version: string) {
    try {
      await invoke<boolean>('set_active_version', { version });
      await loadVersions();
      return { success: true };
    } catch (e) { return { success: false, error: String(e) }; }
  }

  async function loadMirrors() {
    try {
      mirrors.value = await invoke<any[]>('get_mirrors');
    } catch (e) {
      console.error('Failed to load mirrors:', e);
    }
  }

  async function importLocalFrpc() {
    try {
      const selected = await openDialog({
        title: '选择 frpc 压缩包',
        filters: [
          { name: '压缩包', extensions: ['zip'] },
          { name: 'tar.gz', extensions: ['gz'] },
        ],
      });
      if (selected) {
        const path = await invoke<string>('import_local_frpc', { filePath: selected as string });
        await loadVersions();
        return { success: true, path };
      }
      return { success: false, error: 'canceled' };
    } catch (e) { return { success: false, error: String(e) }; }
  }

  // ===== 系统操作 =====
  async function openUrl(url: string) {
    try { await invoke<boolean>('open_url', { url }); } catch (e) { console.error(e); }
  }

  async function relaunchApp() {
    try { await invoke<boolean>('relaunch_app'); } catch (e) { console.error(e); }
  }

  async function openAppData() {
    try { await invoke<boolean>('open_app_data'); } catch (e) { console.error(e); }
  }

  async function checkAppUpdate() {
    try {
      const version = await invoke<string>('check_app_update');
      return version;
    } catch (e) {
      console.error('Failed to check update:', e);
      return null;
    }
  }

  async function loadLocalPorts(): Promise<LocalPort[]> {
    try {
      const ports = await invoke<any[]>('get_local_ports');
      localPorts.value = ports;
      return ports;
    } catch (e) {
      console.error('Failed to load local ports:', e);
      localPorts.value = [];
      return [];
    }
  }

  // ===== 持久化数据管理 =====
  async function savePersistentData() {
    try {
      await invoke<boolean>('save_persistent_data', {
        data: {
          servers: servers.value,
          proxies: proxies.value,
        }
      });
    } catch (e) {
      // 持久化失败必须让用户感知（历史 bug：静默吞错导致数据丢失无感知）
      console.error('Failed to save persistent data:', e);
      addLog({
        timestamp: Date.now(),
        level: 'error',
        message: `⚠️ 数据保存失败，重启后可能丢失：${String(e)}`,
        source: 'app',
      });
    }
  }

  async function loadPersistentData() {
    try {
      const data = await invoke<any>('load_persistent_data');
      if (data && data.servers) {
        servers.value = data.servers || [];
      }
      if (data && data.proxies) {
        proxies.value = data.proxies || [];
      }
    } catch (e) {
      console.error('Failed to load persistent data:', e);
    }
  }

  // ===== 初始化 =====
  let statusTimer: ReturnType<typeof setInterval> | null = null;
  
  function init() {
    applyTheme(theme.value);
    if (statusTimer === null) {
      statusTimer = setInterval(() => {
        refreshProcessStatus();
        refreshServerStatus(); // 刷新所有服务器状态
        refreshProxyStatus();  // 刷新代理状态
        refreshServerTraffic(); // 刷新流量统计
      }, 5000);
    }
    loadSettings();
    loadMirrors();
    loadPersistentData();
    loadTrafficHistory();  // 加载流量历史
    loadConnectionHistory(); // 加载连接历史
  }
  
  // 清理定时器（防止内存泄漏）
  function cleanup() {
    if (statusTimer !== null) {
      clearInterval(statusTimer);
      statusTimer = null;
    }
  }

  return {
    // State
    theme, language,
    autoStart, minimizeToTray, closeToTray, defaultServerId,
    frpConfig, processStatus, logs, frpcLogContent, autoScrollLogs,
    servers, proxies, versions, downloadedVersions, localPorts, mirrors,
    serverStatuses,
    // Getters
    isRunning, frpcStartedAt, liveLogs, appendLiveLog, clearLiveLogs, loadRecentLogsFromDisk, runningServersCount, activeProxies, activeProxiesCount,
    // Settings
    setTheme, setLanguage, loadSettings, saveSettings,
    setDefaultServerId,
    // Config
    loadConfig, saveConfig, resetAllConfig, importTomlConfig,
    // Server/Proxy
    addServer, updateServer, deleteServer,
    addProxy, updateProxy, deleteProxy, modifyProxyStatus,
    // Multi-Server Process Control
    startServer, stopServer, restartServer, refreshServerStatus,
    // Monitoring
    refreshProxyStatus, refreshServerTraffic, getProxyStatus, getServerTraffic, formatTraffic,
    loadTrafficHistory, loadConnectionHistory, restartServerIfRunning,
    // Persistence
    savePersistentData, loadPersistentData,
    // Process
    startFRP, stopFRP, restartFRP, reloadFRP, refreshProcessStatus, detectFrpcProcess,
    // Logs
    addLog, clearLogs, loadFrpcLogContent, openFrpcLogFile,
    // Versions
    loadVersions, loadDownloadedVersions, downloadVersion, deleteVersion,
    setActiveVersion,
    loadMirrors, importLocalFrpc,
    // System
    openUrl, relaunchApp, openAppData, checkAppUpdate, loadLocalPorts,
    // Init
    init,
    cleanup,
  };
});
