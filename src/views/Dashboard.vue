<script setup lang="ts">
/**
 * 概览页
 *
 * 运行时长：基于全局 store 的 frpcStartedAt（后端权威时间戳同步），切页不重置
 * 实时日志：缓冲在全局 store.liveLogs，切页保留；挂载时若为空从磁盘日志预填
 */
import { computed, ref, onMounted, onUnmounted, nextTick, h } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  PlusOutlined, PlayCircleOutlined, PauseCircleOutlined,
  FileTextOutlined, SettingOutlined, ReloadOutlined, ClockCircleOutlined,
} from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';

const appStore = useAppStore();
const router = useRouter();

const autoScroll = ref(true);
const logTerminalRef = ref<HTMLElement | null>(null);
let unlistenLog: UnlistenFn | null = null;

const MAX_LOG_LINES = 500;

// 运行状态计算（基于默认服务器的实时状态）
const status = computed(() => {
  // 优先用默认服务器，回退到第一个服务器（defaultServerId 未设置时）
  const sid = appStore.defaultServerId || appStore.servers[0]?.id;
  const s = sid ? appStore.serverStatuses.get(sid) : null;
  // 后端 ServerStatusResponse 字段：running(布尔) / state("running"/"stopped"等)
  // 历史 bug：前端读 .status（不存在），后端返回 .state → isRunning 永远 false
  const isRunning = s?.running === true || s?.state === 'running';
  return {
    title: '运行状态',
    value: isRunning ? '运行中' : '已停止',
    icon: isRunning ? PlayCircleOutlined : PauseCircleOutlined,
    color: isRunning ? '#52c41a' : '#8c8c8c',
    isRunning,
  };
});

const stats = computed(() => [
  {
    ...status.value,
  },
  {
    title: '运行时长',
    value: formatUptime(uptimeSeconds.value),
    icon: ClockCircleOutlined,
    color: '#1890ff',
  },
  {
    title: '代理总数',
    value: String(appStore.proxies.length),
    icon: FileTextOutlined,
    color: '#722ed1',
  },
  {
    title: '活跃代理',
    value: String(appStore.activeProxiesCount),
    icon: ReloadOutlined,
    color: '#faad14',
  },
]);

function formatUptime(totalSeconds: number): string {
  if (totalSeconds <= 0) return '0秒';
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  if (h > 0) return `${h}时${m}分`;
  if (m > 0) return `${m}分${s}秒`;
  return `${s}秒`;
}

const uptimeSeconds = ref(0);
let uptimeTimer: ReturnType<typeof setInterval> | null = null;

// 运行时长基于全局 store 的启动时刻 + 概览页 status 判定
// （历史 bug：tickUptime 用 appStore.isRunning，但该值基于旧 processStatus，
//  多进程模式永远 false → 运行时长永远 0）
function tickUptime() {
  const startedAt = appStore.frpcStartedAt;
  if (startedAt && status.value.isRunning) {
    uptimeSeconds.value = Math.max(0, Math.floor((Date.now() - startedAt) / 1000));
  } else {
    uptimeSeconds.value = 0;
  }
}

const recentLogs = computed(() => appStore.liveLogs);

function onTerminalScroll() {
  const el = logTerminalRef.value;
  if (!el) return;
  // 距底部 40px 内视为跟随；上滚则暂停自动滚动
  autoScroll.value = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
}

function clearLiveLogs() {
  appStore.clearLiveLogs();
}

// 加载状态（启动/停止按钮动画）
// 历史 bug：此 ref 曾被困在 onUnmounted 闭包内，模板访问到 undefined → 无加载动画
const actionLoading = ref({
  start: false,
  stop: false,
});

async function startFrp() {
  actionLoading.value.start = true;
  try {
    // 优先使用设置的默认服务器，否则回退第一个
    const defaultServer =
      appStore.servers.find((s) => s.id === appStore.defaultServerId) ||
      appStore.servers[0];
    if (!defaultServer) {
      message.warning('请先添加服务器');
      return;
    }
    await appStore.startServer(defaultServer.id);
    message.success('FRP 启动成功');
  } catch (e) {
    message.error(`启动失败：${String(e)}`);
  } finally {
    actionLoading.value.start = false;
  }
}

async function stopFrp() {
  actionLoading.value.stop = true;
  try {
    const defaultServer =
      appStore.servers.find((s) => s.id === appStore.defaultServerId) ||
      appStore.servers[0];
    if (!defaultServer) return;
    await appStore.stopServer(defaultServer.id);
    message.success('FRP 已停止');
  } catch (e) {
    message.error(`停止失败：${String(e)}`);
  } finally {
    actionLoading.value.stop = false;
  }
}

onMounted(async () => {
  // 立即校准一次（切页回来时恢复正确时长，而非从 0 重计）
  tickUptime();
  uptimeTimer = setInterval(tickUptime, 1000);

  // 实时日志缓冲为空时，从磁盘日志预填最近内容（切页/重启后不丢上下文）
  appStore.loadRecentLogsFromDisk();

  // 订阅后端 frpc 实时日志事件
  try {
    unlistenLog = await listen<{ line: string; timestamp: number }>('frpc-log', (event) => {
      appStore.appendLiveLog(event.payload.line, event.payload.timestamp);
      nextTick(() => {
        if (autoScroll.value && logTerminalRef.value) {
          logTerminalRef.value.scrollTop = logTerminalRef.value.scrollHeight;
        }
      });
    });
  } catch (e) {
    console.error('订阅实时日志失败:', e);
  }
});

onUnmounted(() => {
  if (uptimeTimer) clearInterval(uptimeTimer);
  if (unlistenLog) unlistenLog();
});
</script>

<template>
  <div class="dashboard-container">
    <h1 class="page-title">概览</h1>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <div v-for="stat in stats" :key="stat.title" class="stat-card">
        <div class="stat-title">{{ stat.title }}</div>
        <div class="stat-value" :style="{ color: stat.color }">
          <component :is="stat.icon" class="stat-icon" />
          {{ stat.value }}
        </div>
      </div>
    </div>

    <!-- 快速操作 -->
    <a-card class="quick-actions-card" :bordered="false">
      <template #title>快速操作</template>
      <a-space wrap>
        <a-button type="primary" @click="router.push('/servers')">
          <template #icon><PlusOutlined /></template>
          添加服务器
        </a-button>
        <a-button type="primary" :disabled="status.isRunning" :loading="actionLoading.start" @click="startFrp">
          <template #icon><PlayCircleOutlined /></template>
          启动 FRP
        </a-button>
        <a-button danger :disabled="!status.isRunning" :loading="actionLoading.stop" @click="stopFrp">
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

    <!-- 实时日志 -->
    <a-card class="logs-card" :bordered="false">
      <template #title>
        <div class="logs-header">
          <span>实时日志</span>
          <div class="logs-actions">
            <a-checkbox v-model:checked="autoScroll">自动滚动</a-checkbox>
            <a-button size="small" @click="clearLiveLogs">清空</a-button>
            <a-button type="link" size="small" @click="router.push('/logs')">查看全部</a-button>
          </div>
        </div>
      </template>
      <div ref="logTerminalRef" class="log-terminal" @scroll="onTerminalScroll">
        <div v-if="recentLogs.length === 0" class="log-empty">
          暂无运行日志 —— 启动 FRP 后此处将实时显示 frpc 输出
        </div>
        <div v-for="(log, idx) in recentLogs" :key="idx" class="log-line" :class="{ 'log-error': log.isError }">
          <span class="log-ts">{{ log.ts }}</span>
          <span class="log-text">{{ log.line }}</span>
        </div>
      </div>
    </a-card>
  </div>
</template>

<style scoped>
.dashboard-container { padding: 4px 8px; }
.page-title { font-size: 22px; font-weight: 600; margin-bottom: 16px; }
.stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 16px; }
.stat-card { background: #fff; border-radius: 8px; padding: 20px; box-shadow: 0 1px 2px rgba(0,0,0,0.06); }
.stat-title { font-size: 13px; color: #8c8c8c; margin-bottom: 10px; }
.stat-value { font-size: 24px; font-weight: 600; display: flex; align-items: center; gap: 8px; }
.stat-icon { font-size: 22px; }
.quick-actions-card, .logs-card { border-radius: 8px; margin-bottom: 16px; }
.logs-header { display: flex; align-items: center; justify-content: space-between; }
.logs-actions { display: flex; align-items: center; gap: 8px; }
.log-terminal {
  background: #0d1117; color: #c9d1d9; border-radius: 6px;
  padding: 12px; height: 320px; overflow-y: auto;
  font-family: 'Consolas', 'Monaco', monospace; font-size: 12px;
}
.log-empty { color: #6e7681; text-align: center; margin-top: 140px; }
.log-line { padding: 1px 0; word-break: break-all; }
.log-error .log-text { color: #ff7b72; }
.log-ts { color: #6e7681; margin-right: 10px; }
</style>
