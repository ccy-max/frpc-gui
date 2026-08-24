<script setup lang="ts">
import { computed, ref } from 'vue';
import { useAppStore } from '@/stores/app';
import { ReloadOutlined, DeleteOutlined, SearchOutlined } from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';

const appStore = useAppStore();

const searchKeyword = ref('');
const logLevelFilter = ref<string>('all');
const autoScroll = ref(true);

const logLevels = [
  { value: 'all', label: '全部', color: '' },
  { value: 'debug', label: 'Debug', color: 'purple' },
  { value: 'info', label: 'Info', color: 'blue' },
  { value: 'warn', label: 'Warn', color: 'orange' },
  { value: 'error', label: 'Error', color: 'red' },
];

const filteredLogs = computed(() => {
  let logs = appStore.logs;
  
  // 级别过滤
  if (logLevelFilter.value !== 'all') {
    logs = logs.filter(log => log.level === logLevelFilter.value);
  }
  
  // 关键词搜索
  if (searchKeyword.value.trim()) {
    const kw = searchKeyword.value.toLowerCase();
    logs = logs.filter(log => log.message.toLowerCase().includes(kw));
  }
  
  return logs.slice().reverse();
});

function clearLogs() { 
  appStore.clearLogs(); 
  message.success('日志已清空');
}

async function loadFrpcLog() {
  await appStore.loadFrpcLogContent();
  message.success('磁盘日志已加载');
}

function getLevelColor(level: string) {
  const colors: Record<string, string> = {
    debug: '#722ed1',
    info: '#1890ff',
    warn: '#fa8c16',
    error: '#f5222d',
  };
  return colors[level] || '#8c8c8c';
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

    <!-- 过滤工具栏 -->
    <div class="filter-toolbar">
      <a-input
        v-model:value="searchKeyword"
        placeholder="搜索日志内容..."
        style="width: 300px"
        allow-clear
      >
        <template #prefix><SearchOutlined /></template>
      </a-input>
      
      <a-space>
        <span>级别：</span>
        <a-radio-group v-model:value="logLevelFilter" button-style="solid">
          <a-radio-button v-for="level in logLevels" :key="level.value" :value="level.value">
            {{ level.label }}
          </a-radio-button>
        </a-radio-group>
      </a-space>
      
      <a-checkbox v-model:checked="autoScroll">自动滚动</a-checkbox>
    </div>

    <!-- 日志列表 -->
    <a-card class="log-card">
      <div class="log-console" v-if="filteredLogs.length > 0">
        <div v-for="(log, index) in filteredLogs" :key="index" class="log-line">
          <span class="log-time">{{ new Date(log.timestamp).toLocaleString() }}</span>
          <a-tag :color="getLevelColor(log.level)" size="small" class="log-level">
            {{ log.level.toUpperCase() }}
          </a-tag>
          <span class="log-msg">{{ log.message }}</span>
        </div>
      </div>
      <a-empty v-else description="暂无日志" />
    </a-card>

    <!-- FRP 磁盘日志 -->
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

.filter-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
  padding: 12px 16px;
  background: #f5f5f5;
  border-radius: 4px;
}

.log-card {
  :deep(.ant-card-body) { padding: 0; }
}

.log-console {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  background-color: #1e1e1e;
  color: #d4d4d4;
  padding: 16px;
  border-radius: 4px;
  max-height: 500px;
  overflow-y: auto;
}

.log-line {
  padding: 4px 0;
  border-bottom: 1px solid #2d2d2d;
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.log-time {
  color: #858585;
  white-space: nowrap;
}

.log-level {
  min-width: 50px;
  text-align: center;
}

.log-msg {
  color: #d4d4d4;
  flex: 1;
  word-break: break-all;
}
</style>
