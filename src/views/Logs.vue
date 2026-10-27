<script setup lang="ts">
import { computed, ref } from 'vue';
import { useAppStore } from '@/stores/app';
import { ReloadOutlined, DeleteOutlined, SearchOutlined } from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';
import VirtualLogList from '@/components/VirtualLogList.vue';

const appStore = useAppStore();

const searchKeyword = ref('');
const logLevelFilter = ref<string>('all');
const autoScroll = ref(true);
const vlogRef = ref<InstanceType<typeof VirtualLogList>>();

// FRP 磁盘日志行（过滤空行）
const frpcLogLines = computed(() =>
  (appStore.frpcLogContent || '').split('\n').filter(l => l.trim()),
);

// 复制单行日志
async function copyLine(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    message.success('已复制');
  } catch {
    message.error('复制失败');
  }
}

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

    <!-- 日志列表（A3：虚拟滚动，1000+ 行不卡） -->
    <a-card class="log-card">
      <template v-if="filteredLogs.length > 0">
        <VirtualLogList
          ref="vlogRef"
          :items="filteredLogs"
          :item-height="32"
          :height="500"
        >
          <template #item="{ item: log }">
            <div class="log-line" :title="log.message">
              <span class="log-time">{{ new Date(log.timestamp).toLocaleString() }}</span>
              <a-tag :color="getLevelColor(log.level)" size="small" class="log-level">
                {{ log.level.toUpperCase() }}
              </a-tag>
              <span class="log-msg">{{ log.message }}</span>
              <a-tooltip title="复制此行">
                <span class="log-copy" @click="copyLine(log.message)">⧉</span>
              </a-tooltip>
            </div>
          </template>
        </VirtualLogList>
        <div class="log-count-bar">共 {{ filteredLogs.length }} 条</div>
      </template>
      <a-empty v-else description="暂无日志" />
    </a-card>

    <!-- FRP 磁盘日志（同样虚拟化） -->
    <a-card v-if="frpcLogLines.length" title="FRP 磁盘日志" style="margin-top: 16px">
      <VirtualLogList :items="frpcLogLines" :item-height="30" :height="360">
        <template #item="{ item: line }">
          <div class="log-line" :title="line">
            <span class="log-msg">{{ line }}</span>
            <a-tooltip title="复制此行">
              <span class="log-copy" @click="copyLine(line)">⧉</span>
            </a-tooltip>
          </div>
        </template>
      </VirtualLogList>
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.logs-page {
  padding: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;

  .page-title {
    font-size: 24px;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
  }

  .ant-btn {
    border-radius: 8px;
    font-weight: 500;
    transition: all 0.2s ease;

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
    }

    &:active {
      transform: scale(0.98);
    }
  }
}

.filter-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
  padding: 16px;
  background: #f8fafc;
  border-radius: 10px;
  border: 1px solid #e2e8f0;

  .ant-input {
    border-radius: 8px;
  }

  .ant-radio-group {
    .ant-radio-button-wrapper {
      border-radius: 6px;
      margin-right: 4px;
      transition: all 0.2s ease;

      &:hover {
        transform: translateY(-1px);
      }

      &.ant-radio-button-wrapper-checked {
        background: #2563eb;
        border-color: #2563eb;
      }
    }
  }
}

.log-card {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);

  :deep(.ant-card-body) {
    padding: 0;
  }
}

.log-console {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  background-color: #1e293b;
  color: #e2e8f0;
  padding: 16px;
  border-radius: 12px;
  max-height: 500px;
  overflow-y: auto;

  &::-webkit-scrollbar {
    width: 8px;
  }

  &::-webkit-scrollbar-track {
    background: #0f172a;
    border-radius: 4px;
  }

  &::-webkit-scrollbar-thumb {
    background: #475569;
    border-radius: 4px;

    &:hover {
      background: #64748b;
    }
  }
}

// 虚拟列表容器（深色控制台风格）
:deep(.vlog-container) {
  background-color: #1e293b;
  color: #e2e8f0;
  padding: 8px 16px;
  border-radius: 12px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;

  &::-webkit-scrollbar {
    width: 8px;
  }

  &::-webkit-scrollbar-track {
    background: #0f172a;
    border-radius: 4px;
  }

  &::-webkit-scrollbar-thumb {
    background: #475569;
    border-radius: 4px;

    &:hover {
      background: #64748b;
    }
  }
}

.log-count-bar {
  padding: 8px 16px;
  font-size: 12px;
  color: #94a3b8;
  text-align: right;
}

.log-line {
  height: 32px;
  display: flex;
  align-items: center;
  gap: 10px;
  border-bottom: 1px solid #334155;
  overflow: hidden;
}

.log-time {
  color: #94a3b8;
  white-space: nowrap;
  font-size: 12px;
}

.log-level {
  min-width: 55px;
  text-align: center;
  border-radius: 4px;
  font-weight: 500;
  flex-shrink: 0;
}

.log-msg {
  color: #e2e8f0;
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.5;
}

.log-copy {
  flex-shrink: 0;
  color: #64748b;
  cursor: pointer;
  padding: 0 4px;
  opacity: 0;
  transition: opacity 0.15s;

  &:hover {
    color: #e2e8f0;
  }
}

.log-line:hover .log-copy {
  opacity: 1;
}
</style>
