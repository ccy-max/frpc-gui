import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { FrpConfig, ProxyConfig, ProcessStatus, LogEntry } from '@/types';
import { invoke } from '@tauri-apps/api/core';

export const useAppStore = defineStore('app', () => {
  // 应用状态
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
  
  // 计算属性
  const isRunning = computed(() => processStatus.value.running);
  const activeProxiesCount = computed(() => {
    if (!frpConfig.value) return 0;
    return frpConfig.value.proxies.filter(p => p.enabled).length;
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
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  }
  
  async function saveConfig(config: FrpConfig) {
    try {
      const result = await invoke<ConfigResponse>('save_config', { config });
      if (result.success && result.config) {
        frpConfig.value = result.config;
        return { success: true };
      }
      return { success: false, error: result.error };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  }
  
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
  
  // 初始化
  function init() {
    applyTheme(theme.value);
    // 定期刷新进程状态
    setInterval(refreshProcessStatus, 5000);
  }
  
  return {
    theme,
    language,
    frpConfig,
    processStatus,
    logs,
    autoScrollLogs,
    isRunning,
    activeProxiesCount,
    setTheme,
    setLanguage,
    loadConfig,
    saveConfig,
    startFRP,
    stopFRP,
    restartFRP,
    refreshProcessStatus,
    addLog,
    clearLogs,
    init,
  };
});

interface ConfigResponse {
  success: boolean;
  config: FrpConfig | null;
  error: string | null;
}
