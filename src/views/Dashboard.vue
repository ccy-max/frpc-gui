<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';
import { PlusOutlined, PlayCircleOutlined, PauseCircleOutlined, FileTextOutlined, SettingOutlined } from '@ant-design/icons-vue';

const appStore = useAppStore();
const router = useRouter();

const uptimeSeconds = ref(0);
let uptimeTimer: ReturnType<typeof setInterval> | null = null;

const stats = computed(() => [
  { title: '运行状态', value: appStore.isRunning ? '运行中' : '已停止', color: appStore.isRunning ? '#52c41a' : '#8c8c8c' },
  { title: '运行时长', value: formatUptime(uptimeSeconds.value), color: '#1677ff' },
  { title: '代理总数', value: appStore.proxies.length, color: '#722ed1' },
  { title: '活跃代理', value: appStore.activeProxiesCount, color: '#faad14' },
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
  <div>
    <a-page-header title="概览" />
    
    <a-row :gutter="16" style="margin-bottom: 16px">
      <a-col :span="6" v-for="(stat, index) in stats" :key="index">
        <a-statistic :title="stat.title" :value="stat.value" :value-style="{ color: stat.color }" />
      </a-col>
    </a-row>

    <a-divider />

    <a-space wrap style="margin-bottom: 16px">
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

    <a-divider />

    <a-card title="最近日志" style="margin-top: 16px">
      <template #extra>
        <a-button type="link" size="small" @click="router.push('/logs')">查看全部</a-button>
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
