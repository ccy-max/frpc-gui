import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { FrpConfig, ProxyConfig, ProcessStatus, LogEntry } from '@/types';
import { invoke } from '@tauri-apps/api/core';

export const useAppStore = defineStore('app', () => {
  // 应用配置
  const theme = ref<'light' | 'dark' | 'auto'>('auto');
  const language = ref<'zh-CN' | 'en-US'>('zh-CN');
  
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
  const autoScrollLogs = ref(true);
  
  // 服务器列表 (用于 UI 显示)
  const servers = ref<any[]>([]);
  const proxies = ref<any[]>([]);
  const versions = ref<any[]>([]);
  
  // 计算属性
  const isRunning = computed(() => processStatus.value.running);
  
  const runningServersCount = computed(() => {
    return processStatus.value.running ? 1 : 0;
  });
  
  const activeProxies = computed(() => {
    if (!frpConfig.value) return [];
    return frpConfig.value.proxies.filter(p => p.enabled);
  });
  
  const activeProxiesCount = computed(() => {
    return activeProxies.value.length;
  });
  
  // 方法
  function setTheme(newTheme: 'light' | 'dark' | 'auto') {
    theme.value = newTheme;
    applyTheme(newTheme);
  }
  
  function applyTheme(newTheme: 'light' | 'dark' | 'auto') {
    const root = document.documentElement;
    if (newTheme === 'auto') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.classList.toggle('dark', prefersDark);
    } else {
      root.classList.toggle('dark', newTheme === 'dark');
    }
  }
  
  function setLanguage(lang: 'zh-CN' | 'en-US') {
    language.value = lang;
  }
  
  async function loadConfig() {
    try {
      const result = await invoke<ConfigResponse>('load_config');
      if (result.success && result.config) {
        frpConfig.value = result.config;
      }
      return result;
    } catch (error) {
      console.error('Failed to load config:', error);
      return { success: false, config: null, error: String(error) };
    }
  }
  
  async function saveConfig(config: FrpConfig) {
    try {
      const result = await invoke<ConfigResponse>('save_config', { config });
      if (result.success && result.config) {
        frpConfig.value = result.config;
      }
      return result;
    } catch (error) {
      return { success: false, config: null, error: String(error) };
    }
  }
  
  // 服务器管理 (简化版本，直接操作 frpConfig)
  function addServer(serverData: any) {
    servers.value.push(serverData);
  }
  
  function updateServer(id: string, updates: any) {
    const index = servers.value.findIndex(s => s.id === id);
    if (index !== -1) {
      servers.value[index] = { ...servers.value[index], ...updates };
    }
  }
  
  function deleteServer(id: string) {
    servers.value = servers.value.filter(s => s.id !== id);
  }
  
  // 代理管理
  function addProxy(proxyData: any) {
    proxies.value.push(proxyData);
    // 同时更新 frpConfig
    if (frpConfig.value) {
      frpConfig.value.proxies.push(proxyData);
    }
  }
  
  function updateProxy(name: string, updates: any) {
    const index = proxies.value.findIndex(p => p.name === name);
    if (index !== -1) {
      proxies.value[index] = { ...proxies.value[index], ...updates };
    }
    // 同时更新 frpConfig
    if (frpConfig.value) {
      const configIndex = frpConfig.value.proxies.findIndex(p => p.name === name);
      if (configIndex !== -1) {
        frpConfig.value.proxies[configIndex] = { 
          ...frpConfig.value.proxies[configIndex], 
          ...updates 
        };
      }
    }
  }
  
  function deleteProxy(name: string) {
    proxies.value = proxies.value.filter(p => p.name !== name);
    // 同时更新 frpConfig
    if (frpConfig.value) {
      frpConfig.value.proxies = frpConfig.value.proxies.filter(p => p.name !== name);
    }
  }
  
  // FRP 进程控制
  async function startFRP() {
    if (!frpConfig.value) {
      return { success: false, error: '没有配置' };
    }
    
    try {
      await invoke<boolean>('start_frp', { config: frpConfig.value });
      await refreshProcessStatus();
      addLog({
        timestamp: Date.now(),
        level: 'info',
        message: 'FRP 进程已启动',
        source: 'app',
      });
      return { success: true };
    } catch (error) {
      addLog({
        timestamp: Date.now(),
        level: 'error',
        message: `启动 FRP 失败：${error}`,
        source: 'app',
      });
      return { success: false, error: String(error) };
    }
  }
  
  async function stopFRP() {
    try {
      await invoke<boolean>('stop_frp');
      await refreshProcessStatus();
      addLog({
        timestamp: Date.now(),
        level: 'info',
        message: 'FRP 进程已停止',
        source: 'app',
      });
      return { success: true };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  }
  
  async function restartFRP() {
    if (!frpConfig.value) {
      return { success: false, error: '没有配置' };
    }
    
    try {
      await invoke<boolean>('restart_frp', { config: frpConfig.value });
      await refreshProcessStatus();
      addLog({
        timestamp: Date.now(),
        level: 'info',
        message: 'FRP 进程已重启',
        source: 'app',
      });
      return { success: true };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  }
  
  async function refreshProcessStatus() {
    try {
      const status = await invoke<ProcessStatus>('get_process_status');
      processStatus.value = status;
    } catch (error) {
      console.error('Failed to get process status:', error);
    }
  }
  
  // 日志管理
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
  
  // 版本管理
  function addVersion(version: any) {
    versions.value.push(version);
  }
  
  function deleteVersion(version: string) {
    versions.value = versions.value.filter(v => v.version !== version);
  }
  
  // 初始化
  function init() {
    applyTheme(theme.value);
    // 定期刷新进程状态
    setInterval(refreshProcessStatus, 5000);
  }
  
  return {
    // State
    theme,
    language,
    frpConfig,
    processStatus,
    logs,
    autoScrollLogs,
    servers,
    proxies,
    versions,
    
    // Getters
    isRunning,
    runningServersCount,
    activeProxies,
    activeProxiesCount,
    
    // Actions
    setTheme,
    setLanguage,
    loadConfig,
    saveConfig,
    addServer,
    updateServer,
    deleteServer,
    addProxy,
    updateProxy,
    deleteProxy,
    startFRP,
    stopFRP,
    restartFRP,
    refreshProcessStatus,
    addLog,
    clearLogs,
    addVersion,
    deleteVersion,
    init,
  };
});

interface ConfigResponse {
  success: boolean;
  config: FrpConfig | null;
  error: string | null;
}
