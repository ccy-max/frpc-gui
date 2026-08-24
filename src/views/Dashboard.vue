<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';
import { PlusOutlined, PlayCircleOutlined, PauseCircleOutlined, FileTextOutlined, SettingOutlined } from '@ant-design/icons-vue';

const appStore = useAppStore();
const router = useRouter();

const uptimeSeconds = ref(0);
let uptimeTimer: ReturnType<typeof setInterval> | null = null;

const stats = computed(() => [
  { title: '运行状态', value: appStore.isRunning ? '运行中' : '已停止', color: appStore.isRunning ? '#52c41a' : '#8c8c8c', icon: appStore.isRunning ? '●' : '○' },
  { title: '运行时长', value: formatUptime(uptimeSeconds.value), color: '#1677ff', icon: '⏱' },
  { title: '代理总数', value: appStore.proxies.length, color: '#722ed1', icon: '🔌' },
  { title: '活跃代理', value: appStore.activeProxiesCount, color: '#faad14', icon: '⚡' },
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

function startUptimeTimer() {
  if (uptimeTimer) clearInterval(uptimeTimer);
  uptimeTimer = setInterval(() => {
    if (appStore.isRunning) {
      uptimeSeconds.value++;
    }
  }, 1000);
}

onMounted(() => {
  startUptimeTimer();
});

onUnmounted(() => {
  if (uptimeTimer) clearInterval(uptimeTimer);
});

// 监听运行状态变化
const prevRunning = ref(appStore.isRunning);
const watcher = computed(() => appStore.isRunning);
watch(watcher, (newVal) => {
  if (newVal !== prevRunning.value) {
    if (newVal) {
      uptimeSeconds.value = 0; // 重新开始计时
    }
    prevRunning.value = newVal;
  }
});

const recentLogs = computed(() => appStore.logs.slice(-5).reverse());

const columns = [
  { title: '时间', dataIndex: 'timestamp', key: 'timestamp', width: 180,
    customRender: ({ text }: { text: number }) => new Date(text).toLocaleString() },
  { title: '级别', key: 'level', width: 80,
    customRender: ({ record }: { record: any }) => {
      const colors: Record<string, string> = { debug: 'purple', info: 'blue', warn: 'orange', error: 'red' };
      return h('a-tag', { color: colors[record.level] || '' }, { default: () => record.level.toUpperCase() });
    }
  },
  { title: '消息', dataIndex: 'message', key: 'message' },
];

// 简单的 h 函数
function h(tag: string, props: any = {}, children: any = {}) {
  return { tag, props, children };
}
</script>

<template>
  <div class="dashboard">
    <h2 class="page-title">概览</h2>

    <a-row :gutter="16" class="stats-row">
      <a-col :span="6" v-for="(stat, index) in stats" :key="index">
        <a-card :bordered="true">
          <div class="stat-content">
            <div class="stat-icon">{{ stat.icon }}</div>
            <div class="stat-value" :style="{ color: stat.color }">{{ stat.value }}</div>
            <div class="stat-label">{{ stat.title }}</div>
          </div>
        </a-card>
      </a-col>
    </a-row>

    <a-card title="快速启动" style="margin-bottom: 16px">
      <a-space size="middle" wrap>
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

    <a-card title="最近日志">
      <template #extra>
        <a-button type="link" size="small" @click="router.push('/logs')">查看全部</a-button>
      </template>
      <a-empty v-if="recentLogs.length === 0" description="暂无日志" />
      <a-table v-else :data-source="recentLogs" :columns="columns as any" :pagination="false" size="small" />
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.dashboard { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-title { font-size: 24px; font-weight: 600; margin-bottom: 24px; }
.stats-row { margin-bottom: 24px; }
.stat-content { text-align: center; }
.stat-icon { font-size: 32px; margin-bottom: 8px; }
.stat-value { font-size: 28px; font-weight: 700; line-height: 1; margin-bottom: 4px; }
.stat-label { font-size: 13px; color: #8c8c8c; }
</style>
