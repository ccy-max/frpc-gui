<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';
import { PlusOutlined, PlayCircleOutlined, FileTextOutlined, SettingOutlined } from '@ant-design/icons-vue';

const appStore = useAppStore();
const router = useRouter();

const stats = computed(() => [
  { title: '运行中的服务器', value: appStore.runningServersCount, color: '#52c41a' },
  { title: '代理总数', value: appStore.proxies.length, color: '#1677ff' },
  { title: '活跃代理', value: appStore.activeProxiesCount, color: '#faad14' },
  { title: 'FRP 版本', value: appStore.versions.length, color: '#8c8c8c' },
]);

const recentLogs = computed(() => appStore.logs.slice(-5).reverse());

const columns = [
  { title: '时间', dataIndex: 'timestamp', key: 'timestamp', width: 180,
    customRender: ({ text }: { text: number }) => new Date(text).toLocaleString() },
  { title: '级别', dataIndex: 'level', key: 'level', width: 80 },
  { title: '消息', dataIndex: 'message', key: 'message' },
];
</script>

<template>
  <div class="dashboard">
    <h2 class="page-title">概览</h2>

    <a-row :gutter="16" class="stats-row">
      <a-col :span="6" v-for="(stat, index) in stats" :key="index">
        <a-card :bordered="true">
          <div class="stat-content">
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
        <a-button type="primary" @click="appStore.startFRP()">
          <template #icon><PlayCircleOutlined /></template>
          启动 FRP
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
      <a-table v-else :data-source="recentLogs" :columns="columns" :pagination="false" size="small" />
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.dashboard { padding: 24px; height: calc(100vh - 60px); overflow-y: auto; }
.page-title { font-size: 24px; font-weight: 600; margin-bottom: 24px; }
.stats-row { margin-bottom: 24px; }
.stat-content { text-align: center; }
.stat-value { font-size: 28px; font-weight: 700; line-height: 1; margin-bottom: 4px; }
.stat-label { font-size: 13px; color: #8c8c8c; }
</style>
