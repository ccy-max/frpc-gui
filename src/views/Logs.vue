<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useAppStore } from '@/stores/app';
import { ReloadOutlined, DeleteOutlined } from '@ant-design/icons-vue';

const appStore = useAppStore();
const logs = computed(() => appStore.logs.slice().reverse());

function clearLogs() { appStore.clearLogs(); }

async function loadFrpcLog() {
  await appStore.loadFrpcLogContent();
}
</script>

<template>
  <div class="logs-page">
    <div class="page-header">
      <h2 class="page-title">日志查看</h2>
      <a-space>
        <a-button @click="loadFrpcLog">
          <template #icon><ReloadOutlined /></template>
          加载磁盘日志
        </a-button>
        <a-button danger @click="clearLogs">
          <template #icon><DeleteOutlined /></template>
          清空日志
        </a-button>
      </a-space>
    </div>

    <a-card>
      <div class="log-console" v-if="logs.length > 0">
        <div v-for="(log, index) in logs" :key="index" class="log-line">
          <span class="log-time">{{ new Date(log.timestamp).toLocaleString() }}</span>
          <span :class="['log-level', 'log-' + log.level]">[{{ log.level.toUpperCase() }}]</span>
          <span class="log-msg">{{ log.message }}</span>
        </div>
      </div>
      <a-empty v-else description="暂无日志" />
    </a-card>

    <a-card v-if="appStore.frpcLogContent" title="FRP 磁盘日志" style="margin-top: 16px">
      <div class="log-console">
        <div v-for="(line, i) in appStore.frpcLogContent.split('\n').filter(l => l.trim())" :key="i" class="log-line">
          <span class="log-msg">{{ line }}</span>
        </div>
      </div>
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.logs-page { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.page-title { font-size: 24px; font-weight: 600; }
.log-console { font-family: 'Consolas', 'Monaco', monospace; font-size: 13px; background-color: #1e1e1e; color: #d4d4d4; padding: 16px; border-radius: 4px; max-height: 400px; overflow-y: auto; }
.log-line { padding: 4px 0; border-bottom: 1px solid #2d2d2d; }
.log-time { color: #858585; margin-right: 8px; }
.log-level { margin-right: 8px; font-weight: 600; }
.log-info { color: #4fc3f7; }
.log-warn { color: #ffb74d; }
.log-error { color: #ef5350; }
.log-debug { color: #9575cd; }
.log-msg { color: #d4d4d4; }
</style>
