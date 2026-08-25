import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { FrpConfig, ProcessStatus, LogEntry } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';

export const useAppStore = defineStore('app', () => {
  // 应用设置（持久化）
  const theme = ref<'light' | 'dark' | 'auto'>('auto');
  const language = ref<'zh-CN' | 'en-US'>('zh-CN');
  const frpcPath = ref('');
  const configPath = ref('');
  const logPath = ref('');
  const autoStart = ref(false);
  const minimizeToTray = ref(true);
  const closeToTray = ref(true);

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

  // 计算属性
  const isRunning = computed(() => processStatus.value.running);
  const runningServersCount = computed(() => processStatus.value.running ? 1 : 0);
  const activeProxies = computed(() => {
    if (!frpConfig.value) return [];
    return frpConfig.value.proxies.filter(p => p.enabled);
  });
  const activeProxiesCount = computed(() => activeProxies.value.length);

  // ===== 设置持久化 =====
  async function loadSettings() {
    try {
      const s = await invoke<any>('load_settings');
      theme.value = s.theme || 'auto';
      language.value = s.language || 'zh-CN';
      frpcPath.value = s.frpc_path || '';
      configPath.value = s.config_path || '';
      logPath.value = s.log_path || '';
      autoStart.value = s.auto_start || false;
      minimizeToTray.value = s.minimize_to_tray ?? true;
      closeToTray.value = s.close_to_tray ?? true;
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
          frpc_path: frpcPath.value,
          config_path: configPath.value,
          log_path: logPath.value,
          auto_start: autoStart.value,
          minimize_to_tray: minimizeToTray.value,
          close_to_tray: closeToTray.value,
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

  // ===== 文件选择 =====
  async function pickFrpcPath() {
    try {
      const selected = await openDialog({
        title: '选择 frpc 可执行文件',
        filters: [{ name: '可执行文件', extensions: ['exe'] }],
      });
      if (selected) {
        frpcPath.value = selected as string;
        await saveSettings();
      }
    } catch (e) {
      console.error('Failed to pick frpc path:', e);
    }
  }

  async function pickConfigPath() {
    try {
      const selected = await saveDialog({
        title: '选择配置文件保存位置',
        defaultPath: 'frpc.toml',
        filters: [{ name: 'TOML 配置', extensions: ['toml'] }],
      });
      if (selected) {
        configPath.value = selected as string;
        await saveSettings();
      }
    } catch (e) {
      console.error('Failed to pick config path:', e);
    }
  }

  async function pickLogPath() {
    try {
      const selected = await openDialog({
        title: '选择日志目录',
        directory: true,
      });
      if (selected) {
        logPath.value = selected as string;
        await saveSettings();
      }
    } catch (e) {
      console.error('Failed to pick log path:', e);
    }
  }

  // ===== 配置管理 =====
  async function loadConfig() {
    try {
      const result = await invoke<any>('load_config');
      if (result.success && result.config) {
        frpConfig.value = result.config;
        proxies.value = result.config.proxies || [];
      }
      return result;
    } catch (e) {
      return { success: false, config: null, error: String(e) };
    }
  }

  async function saveConfig(config: FrpConfig) {
    try {
      // 转换端口为字符串（frpc TOML 期望字符串）
      const configToSave = {
        ...config,
        server_port: String(config.server_port),
        proxies: config.proxies?.map(p => ({
          ...p,
          local_port: p.local_port ? String(p.local_port) : undefined,
          remote_port: p.remote_port ? String(p.remote_port) : undefined,
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
      frpcPath.value = '';
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
        frpConfig.value = config;
        proxies.value = config.proxies || [];
        return { success: true };
      }
      return { success: false, error: 'canceled' };
    } catch (e) {
      return { success: false, error: String(e) };
    }
  }

  // ===== 服务器管理 =====
  function addServer(data: any) { servers.value.push(data); }
  function updateServer(id: string, updates: any) {
    const i = servers.value.findIndex(s => s.id === id);
    if (i !== -1) servers.value[i] = { ...servers.value[i], ...updates };
  }
  function deleteServer(id: string) { servers.value = servers.value.filter(s => s.id !== id); }

  // ===== 代理管理 =====
  function addProxy(data: any) {
    proxies.value.push(data);
    if (frpConfig.value) frpConfig.value.proxies.push(data);
  }
  function updateProxy(name: string, updates: any) {
    const i = proxies.value.findIndex(p => p.name === name);
    if (i !== -1) proxies.value[i] = { ...proxies.value[i], ...updates };
    if (frpConfig.value) {
      const ci = frpConfig.value.proxies.findIndex(p => p.name === name);
      if (ci !== -1) frpConfig.value.proxies[ci] = { ...frpConfig.value.proxies[ci], ...updates };
    }
  }
  function deleteProxy(name: string) {
    proxies.value = proxies.value.filter(p => p.name !== name);
    if (frpConfig.value) frpConfig.value.proxies = frpConfig.value.proxies.filter(p => p.name !== name);
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

  // ===== FRP 进程控制 =====
  async function startFRP() {
    if (!frpConfig.value) return { success: false, error: '没有配置' };
    try {
      await invoke<boolean>('start_frp', { config: frpConfig.value });
      await refreshProcessStatus();
      addLog({ timestamp: Date.now(), level: 'info', message: 'FRP 进程已启动', source: 'app' });
      return { success: true };
    } catch (e) {
      addLog({ timestamp: Date.now(), level: 'error', message: `启动 FRP 失败：${e}`, source: 'app' });
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
      await invoke<boolean>('restart_frp', { config: frpConfig.value });
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
      frpcPath.value = path;
      await saveSettings();
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
        frpcPath.value = path;
        await saveSettings();
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

  // ===== 初始化 =====
  let statusTimer: ReturnType<typeof setInterval> | null = null;
  function init() {
    applyTheme(theme.value);
    if (statusTimer === null) {
      statusTimer = setInterval(refreshProcessStatus, 5000);
    }
    loadSettings();
    loadMirrors();
  }

  return {
    // State
    theme, language, frpcPath, configPath, logPath,
    autoStart, minimizeToTray, closeToTray,
    frpConfig, processStatus, logs, frpcLogContent, autoScrollLogs,
    servers, proxies, versions, downloadedVersions, localPorts, mirrors,
    // Getters
    isRunning, runningServersCount, activeProxies, activeProxiesCount,
    // Settings
    setTheme, setLanguage, loadSettings, saveSettings,
    pickFrpcPath, pickConfigPath, pickLogPath,
    // Config
    loadConfig, saveConfig, resetAllConfig, importTomlConfig,
    // Server/Proxy
    addServer, updateServer, deleteServer,
    addProxy, updateProxy, deleteProxy, modifyProxyStatus,
    // Process
    startFRP, stopFRP, restartFRP, reloadFRP, refreshProcessStatus, detectFrpcProcess,
    // Logs
    addLog, clearLogs, loadFrpcLogContent, openFrpcLogFile,
    // Versions
    loadVersions, loadDownloadedVersions, downloadVersion, deleteVersion,
    loadMirrors, importLocalFrpc,
    // System
    openUrl, relaunchApp, openAppData, checkAppUpdate, loadLocalPorts,
    // Init
    init,
  };
});
