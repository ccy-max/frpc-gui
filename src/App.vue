<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { theme as antdThemeApi } from 'ant-design-vue';
import zhCN from 'ant-design-vue/es/locale/zh_CN';
import enUS from 'ant-design-vue/es/locale/en_US';
import { useAppStore } from '@/stores/app';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { confirm } from '@tauri-apps/plugin-dialog';

const appStore = useAppStore();

// antd 主题联动（A1）：dark class 只影响自绘样式，
// ConfigProvider 才让 Modal/Dropdown/message/DatePicker 等全部组件跟随
const antdTheme = computed(() => ({
  algorithm: appStore.isDarkMode ? antdThemeApi.darkAlgorithm : antdThemeApi.defaultAlgorithm,
  token: {
    colorPrimary: '#2563eb',
    borderRadius: 8,
  },
}));
const antdLocale = computed(() => (appStore.language === 'en-US' ? enUS : zhCN));

// 拦截右键菜单和刷新快捷键（防止 WebView 重新加载导致内存状态全部丢失）
// 生产环境禁用；开发环境保留右键方便调试
function onContextMenu(e: MouseEvent) {
  if (import.meta.env.PROD) {
    e.preventDefault();
  }
}

function onKeyDown(e: KeyboardEvent) {
  // F12 - 开启开发者工具（生产环境也放行，方便调试）
  if (e.key === 'F12') {
    return; // 不拦截，让 WebView 默认处理
  }
  // F5 / Ctrl+R / Ctrl+Shift+R / Cmd+R（Mac） - 阻止页面刷新
  if (e.key === 'F5' ||
      (e.ctrlKey && e.key.toLowerCase() === 'r') ||
      (e.metaKey && e.key.toLowerCase() === 'r')) {
    e.preventDefault();
  }
  // Ctrl+Shift+I - 开启开发者工具（备用快捷键）
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'i') {
    return; // 不拦截
  }
}

onMounted(async () => {
  document.addEventListener('contextmenu', onContextMenu);
  document.addEventListener('keydown', onKeyDown);

  appStore.init();
  // 延迟加载配置，避免阻塞渲染
  setTimeout(() => {
    appStore.loadConfig().catch(console.error);
  }, 100);

  // 拦截窗口关闭按钮：弹选择对话框
  // 后端 CloseRequested 事件已阻止默认关闭，等前端决定
  await listen('window-close-requested', async () => {
    const confirmed = await confirm(
      '关闭时停止 FRP 进程？\n\n✓ 停止并退出\n✗ 最小化到托盘',
      '确认退出'
    );
    if (confirmed) {
      // 用户选择退出：杀 frpc + 直接退出（不调 window.close() 避免死锁）
      await invoke('exit_app').catch(console.error);
    } else {
      // 用户选择最小化：保持窗口隐藏
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      getCurrentWindow().hide();
    }
  });

  // 实时日志：监听后端 frpc-log 事件（frpc stdout → log channel → emit → 前端）
  // 历史 bug：后端正确 emit 了 frpc-log 事件，但前端从未 listen → 实时日志永远空白
  await listen('frpc-log', (event) => {
    const payload = event.payload as { line: string; timestamp: number };
    appStore.appendLiveLog(payload.line, payload.timestamp);
  });

  // 托盘"退出"菜单也走同样流程
  await listen('app-quit-requested', async () => {
    const confirmed = await confirm(
      '确定要退出 FRPC GUI 吗？\n\n正在运行的 FRP 进程将被终止。',
      '确认退出'
    );
    if (confirmed) {
      await invoke('exit_app').catch(console.error);
    }
  });
});

onUnmounted(() => {
  document.removeEventListener('contextmenu', onContextMenu);
  document.removeEventListener('keydown', onKeyDown);
  appStore.cleanup();
});
</script>

<template>
  <a-config-provider :theme="antdTheme" :locale="antdLocale">
    <router-view v-slot="{ Component }">
      <transition name="page" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </a-config-provider>
</template>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

/* 路由切换过渡（C3）：fade + 轻微上移，尊重 reduced-motion */
.page-enter-active, .page-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}
.page-enter-from { opacity: 0; transform: translateY(8px); }
.page-leave-to { opacity: 0; }
@media (prefers-reduced-motion: reduce) {
  .page-enter-active, .page-leave-active { transition: none; }
}
</style>
