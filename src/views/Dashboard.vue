<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch, h } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';
import {
  PlusOutlined, PlayCircleOutlined, PauseCircleOutlined,
  CheckCircleOutlined, ClockCircleOutlined,
  CloudServerOutlined, ThunderboltOutlined,
  FileTextOutlined, SettingOutlined
} from '@ant-design/icons-vue';

const appStore = useAppStore();
const router = useRouter();

const uptimeSeconds = ref(0);
let uptimeTimer: ReturnType<typeof setInterval> | null = null;

const stats = computed(() => [
  {
    title: '运行状态',
    value: appStore.isRunning ? '运行中' : '已停止',
    prefix: () => h(CheckCircleOutlined),
    valueStyle: { color: appStore.isRunning ? '#10b981' : '#64748b' }
  },
  {
    title: '运行时长',
    value: formatUptime(uptimeSeconds.value),
    prefix: () => h(ClockCircleOutlined),
    valueStyle: { color: '#2563eb' }
  },
  {
    title: '代理总数',
    value: appStore.proxies.length,
    prefix: () => h(CloudServerOutlined),
    valueStyle: { color: '#7c3aed' }
  },
  {
    title: '活跃代理',
    value: appStore.activeProxiesCount,
    prefix: () => h(ThunderboltOutlined),
    valueStyle: { color: '#f59e0b' }
  },
]);

function formatUptime(seconds: number): string {
  if (seconds < 60) return `${seconds}秒`;
  if (seconds < 3600) {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}分${s}秒`;
  }
  if (seconds < 86400) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    return `${h}时${m}分`;
  }
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  return `${d}天${h}时`;
}

onMounted(() => {
  uptimeTimer = setInterval(() => {
    if (appStore.isRunning) uptimeSeconds.value++;
  }, 1000);
});

onUnmounted(() => {
  if (uptimeTimer) clearInterval(uptimeTimer);
});

const prevRunning = ref(appStore.isRunning);
watch(() => appStore.isRunning, (newVal) => {
  if (newVal !== prevRunning.value && newVal) uptimeSeconds.value = 0;
  prevRunning.value = newVal;
});

const recentLogs = computed(() => appStore.logs.slice(-5).reverse());
const logColors: Record<string, string> = { debug: 'purple', info: 'blue', warn: 'orange', error: 'red' };
</script>

<template>
  <div class="dashboard-container">
    <div class="page-header">
      <h1 class="page-title">概览</h1>
    </div>

    <a-row :gutter="[16, 16]" style="margin-bottom: 24px">
      <a-col :xs="24" :sm="12" :lg="6" v-for="(stat, index) in stats" :key="index">
        <a-card :bordered="false" class="stat-card">
          <a-statistic
            :title="stat.title"
            :value="stat.value"
            :value-style="stat.valueStyle"
          >
            <template #prefix>
              <component :is="stat.prefix" />
            </template>
          </a-statistic>
        </a-card>
      </a-col>
    </a-row>

    <a-card title="快速操作" class="action-card" style="margin-bottom: 24px">
      <a-space wrap>
        <a-button type="primary" @click="router.push('/servers')">
          <template #icon><PlusOutlined /></template>
          添加服务器
        </a-button>
        <a-button type="primary" @click="appStore.startFRP()" :disabled="appStore.isRunning">
          <template #icon><PlayCircleOutlined /></template>
          启动 FRP
        </a-button>
        <a-button danger ghost @click="appStore.stopFRP()" :disabled="!appStore.isRunning">
          <template #icon><PauseCircleOutlined /></template>
          停止 FRP
        </a-button>
        <a-button @click="router.push('/logs')">
          <template #icon><FileTextOutlined /></template>
          查看日志
        </a-button>
        <a-button @click="router.push('/settings')">
          <template #icon><SettingOutlined /></template>
          设置
        </a-button>
      </a-space>
    </a-card>

    <a-card title="最近日志" class="logs-card">
      <template #extra>
        <a-button type="link" @click="router.push('/logs')">查看全部</a-button>
      </template>
      <a-empty v-if="recentLogs.length === 0" description="暂无日志" />
      <a-table v-else :data-source="recentLogs" :pagination="false" size="small">
        <a-table-column title="时间" dataIndex="timestamp" width="180">
          <template #bodyCell="{ record }">
            {{ new Date(record.timestamp).toLocaleString() }}
          </template>
        </a-table-column>
        <a-table-column title="级别" key="level" width="80">
          <template #bodyCell="{ record }">
            <a-tag :color="logColors[record.level] || ''">
              {{ record.level.toUpperCase() }}
            </a-tag>
          </template>
        </a-table-column>
        <a-table-column title="消息" dataIndex="message" />
      </a-table>
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.dashboard-container {
  padding: 24px;
}

.page-header {
  margin-bottom: 24px;

  .page-title {
    font-size: 24px;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
  }
}

.stat-card {
  background: #ffffff;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  transition: all 0.2s ease;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
  }

  :deep(.ant-statistic-title) {
    font-size: 14px;
    color: #64748b;
    font-weight: 500;
  }

  :deep(.ant-statistic-content) {
    font-size: 28px;
    font-weight: 700;
  }

  :deep(.ant-statistic .anticon) {
    font-size: 20px;
  }
}

.action-card {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);

  :deep(.ant-card-head) {
    font-weight: 600;
    font-size: 16px;
    color: #1e293b;
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

.logs-card {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);

  :deep(.ant-card-head) {
    font-weight: 600;
    font-size: 16px;
    color: #1e293b;
  }
}
</style>
