<script setup lang="ts">
import { onMounted } from 'vue';
import { useAppStore } from '@/stores/app';

const appStore = useAppStore();

onMounted(async () => {
  appStore.init();
  // 延迟加载配置，避免阻塞渲染
  setTimeout(() => {
    appStore.loadConfig().catch(console.error);
  }, 100);
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
