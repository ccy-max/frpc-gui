<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { Refresh, Delete } from '@element-plus/icons-vue';

const appStore = useAppStore();

const logs = computed(() => appStore.logs.slice().reverse());

function clearLogs() {
  appStore.clearLogs();
}
</script>

<template>
  <div class="logs-page">
    <div class="page-header">
      <h2 class="page-title">日志查看</h2>
      <div class="actions">
        <el-button :icon="Refresh" @click="logs">刷新</el-button>
        <el-button type="danger" :icon="Delete" @click="clearLogs">清空日志</el-button>
      </div>
    </div>

    <el-card>
      <div class="log-console" v-if="logs.length > 0">
        <div v-for="(log, index) in logs" :key="index" class="log-line">
          <span class="log-time">{{ new Date(log.timestamp).toLocaleString() }}</span>
          <span :class="['log-level', 'log-' + log.level]">[{{ log.level.toUpperCase() }}]</span>
          <span class="log-msg">{{ log.message }}</span>
        </div>
      </div>
      <el-empty v-else description="暂无日志" />
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.logs-page {
  padding: 24px;
  height: calc(100vh - 60px);
  overflow-y: auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
}

.actions {
  display: flex;
  gap: 8px;
}

.log-console {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  background-color: #1e1e1e;
  color: #d4d4d4;
  padding: 16px;
  border-radius: 4px;
  height: calc(100vh - 200px);
  overflow-y: auto;
}

.log-line {
  padding: 4px 0;
  border-bottom: 1px solid #2d2d2d;
}

.log-time {
  color: #858585;
  margin-right: 8px;
}

.log-level {
  margin-right: 8px;
  font-weight: 600;
}

.log-info { color: #4fc3f7; }
.log-warn { color: #ffb74d; }
.log-error { color: #ef5350; }
.log-debug { color: #9575cd; }

.log-msg {
  color: #d4d4d4;
}
</style>
