import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ServerConfig, ProxyConfig, AppConfig, LogEntry, FrpVersion, ProcessStatus } from '@/types';

export const useAppStore = defineStore('app', () => {
  // 应用配置
  const config = ref<AppConfig>({
    language: 'zh-CN',
    theme: 'auto',
    frpBinaryPath: '',
    configPath: '',
    logPath: '',
    autoStart: false,
    minimizeToTray: true,
    closeToTray: true,
    checkUpdateOnStart: true,
    downloadMirror: 'auto',
  });

  // 服务器列表
  const servers = ref<ServerConfig[]>([]);
  
  // 代理列表
  const proxies = ref<ProxyConfig[]>([]);
  
  // FRP 版本列表
  const versions = ref<FrpVersion[]>([]);
  
  // 当前选中的服务器 ID
  const selectedServerId = ref<string | null>(null);
  
  // 日志列表
  const logs = ref<LogEntry[]>([]);
  
  // 进程状态
  const processStatus = ref<Record<string, ProcessStatus>>({});

  // 计算属性
  const currentServer = computed(() => 
    servers.value.find(s => s.id === selectedServerId.value)
  );

  const activeProxies = computed(() => 
    proxies.value.filter(p => p.enabled)
  );

  const runningServersCount = computed(() => 
    servers.value.filter(s => {
      const status = processStatus.value[s.id];
      return status?.running;
    }).length
  );

  // 方法
  function setLanguage(lang: 'zh-CN' | 'en-US') {
    config.value.language = lang;
    // 这里可以同步到后端
  }

  function setTheme(theme: 'light' | 'dark' | 'auto') {
    config.value.theme = theme;
    applyTheme(theme);
  }

  function applyTheme(theme: 'light' | 'dark' | 'auto') {
    const root = document.documentElement;
    if (theme === 'auto') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.classList.toggle('dark', prefersDark);
    } else {
      root.classList.toggle('dark', theme === 'dark');
    }
  }

  function addServer(server: ServerConfig) {
    servers.value.push(server);
  }

  function updateServer(id: string, updates: Partial<ServerConfig>) {
    const index = servers.value.findIndex(s => s.id === id);
    if (index !== -1) {
      servers.value[index] = { ...servers.value[index], ...updates, updatedAt: Date.now() };
    }
  }

  function deleteServer(id: string) {
    servers.value = servers.value.filter(s => s.id !== id);
  }

  function addProxy(proxy: ProxyConfig) {
    proxies.value.push(proxy);
  }

  function updateProxy(id: string, updates: Partial<ProxyConfig>) {
    const index = proxies.value.findIndex(p => p.name === id);
    if (index !== -1) {
      proxies.value[index] = { ...proxies.value[index], ...updates, updatedAt: Date.now() };
    }
  }

  function deleteProxy(name: string) {
    proxies.value = proxies.value.filter(p => p.name !== name);
  }

  function addLog(entry: LogEntry) {
    logs.value.push(entry);
    // 限制日志数量
    if (logs.value.length > 1000) {
      logs.value = logs.value.slice(-500);
    }
  }

  function clearLogs() {
    logs.value = [];
  }

  function updateProcessStatus(serverId: string, status: ProcessStatus) {
    processStatus.value[serverId] = status;
  }

  function addVersion(version: FrpVersion) {
    const exists = versions.value.find(v => v.version === version.version);
    if (!exists) {
      versions.value.push(version);
    }
  }

  function deleteVersion(version: string) {
    versions.value = versions.value.filter(v => v.version !== version);
  }

  // 加载配置
  async function loadConfig() {
    // TODO: 从后端加载配置
    try {
      // const loaded = await invoke('load_config');
      // config.value = loaded;
      applyTheme(config.value.theme);
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  }

  // 保存配置
  async function saveConfig() {
    // TODO: 保存到后端
    try {
      // await invoke('save_config', { config: config.value });
    } catch (error) {
      console.error('Failed to save config:', error);
    }
  }

  return {
    config,
    servers,
    proxies,
    versions,
    selectedServerId,
    logs,
    processStatus,
    currentServer,
    activeProxies,
    runningServersCount,
    setLanguage,
    setTheme,
    addServer,
    updateServer,
    deleteServer,
    addProxy,
    updateProxy,
    deleteProxy,
    addLog,
    clearLogs,
    updateProcessStatus,
    addVersion,
    deleteVersion,
    loadConfig,
    saveConfig,
  };
});
