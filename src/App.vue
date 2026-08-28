<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { confirm } from '@tauri-apps/plugin-dialog';

const appStore = useAppStore();

// 拦截右键菜单和刷新快捷键（防止 WebView 重新加载导致内存状态全部丢失）
// 生产环境禁用；开发环境保留右键方便调试
function onContextMenu(e: MouseEvent) {
  if (import.meta.env.PROD) {
    e.preventDefault();
  }
}

function onKeyDown(e: KeyboardEvent) {
  // F5 / Ctrl+R / Ctrl+Shift+R / Cmd+R（Mac）
  if (e.key === 'F5' ||
      (e.ctrlKey && e.key.toLowerCase() === 'r') ||
      (e.metaKey && e.key.toLowerCase() === 'r')) {
    e.preventDefault();
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
      // 用户选择退出：杀 frpc + 通知后端真正退出
      await invoke('kill_all_frpc_on_exit').catch(console.error);
      // 发送 quit 事件让后端执行 app.exit(0)
      window.close();
    } else {
      // 用户选择最小化：保持窗口隐藏
      // 后端已 prevent_close，前端只需隐藏窗口
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      getCurrentWindow().hide();
    }
  });

  // 托盘"退出"菜单也走同样流程
  await listen('app-quit-requested', async () => {
    const confirmed = await confirm(
      '确定要退出 FRPC GUI 吗？\n\n正在运行的 FRP 进程将被终止。',
      '确认退出'
    );
    if (confirmed) {
      await invoke('kill_all_frpc_on_exit').catch(console.error);
      window.close();
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
  <router-view />
</template>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>
