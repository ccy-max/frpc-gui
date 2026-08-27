<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useAppStore } from '@/stores/app';

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
