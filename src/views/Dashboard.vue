<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch, h } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';
import {
  PlusOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
  FileTextOutlined,
  SettingOutlined,
  CheckCircleOutlined,
  ClockCircleOutlined,
  CloudServerOutlined,
  ThunderboltOutlined
} from '@ant-design/icons-vue';

const appStore = useAppStore();
const router = useRouter();

const uptimeSeconds = ref(0);
let uptimeTimer: ReturnType<typeof setInterval> | null = null;

const stats = computed(() => [
  {
    title: '运行状态',
    value: appStore.isRunning ? '运行中' : '已停止',
    prefix: h(CheckCircleOutlined),
    valueStyle: { color: appStore.isRunning ? '#52c41a' : '#8c8c8c' }
  },
  {
    title: '运行时长',
    value: formatUptime(uptimeSeconds.value),
    prefix: h(ClockCircleOutlined),
    valueStyle: { color: '#1677ff' }
  },
  {
    title: '代理总数',
    value: appStore.proxies.length,
    prefix: h(CloudServerOutlined),
    valueStyle: { color: '#722ed1' }
  },
  {
    title: '活跃代理',
    value: appStore.activeProxiesCount,
    prefix: h(ThunderboltOutlined),
    valueStyle: { color: '#faad14' }
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
    <!-- 统计卡片 -->
    <a-row :gutter="[16, 16]" style="margin-bottom: 24px">
      <a-col :span="6" v-for="(stat, index) in stats" :key="index">
        <a-card :bordered="false" class="stat-card">
          <a-statistic
            :title="stat.title"
            :value="stat.value"
            :value-style="stat.valueStyle"
            :prefix="stat.prefix.component"
          />
        </a-card>
      </a-col>
    </a-row>

    <!-- 快速操作 -->
    <a-card title="快速操作" style="margin-bottom: 24px">
      <a-space wrap>
        <a-button type="primary" @click="router.push('/servers')">
          <template #icon><PlusOutlined /></template>
          添加服务器
        </a-button>
        <a-button type="primary" @click="appStore.startFRP()" :disabled="appStore.isRunning">
          <template #icon><PlayCircleOutlined /></template>
          启动 FRP
        </a-button>
        <a-button danger @click="appStore.stopFRP()" :disabled="!appStore.isRunning">
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

    <!-- 最近日志 -->
    <a-card title="最近日志">
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

<style scoped>
.dashboard-container {
  padding: 24px;
}

.stat-card {
  text-align: center;
}

.stat-card :deep(.ant-statistic-title) {
  font-size: 14px;
  color: rgba(0, 0, 0, 0.45);
}

.stat-card :deep(.ant-statistic-content) {
  font-size: 24px;
  font-weight: 600;
}
</style>
