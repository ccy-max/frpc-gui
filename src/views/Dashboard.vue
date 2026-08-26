<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, nextTick, h } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
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

// 运行时长基于全局 store 的启动时刻计算
// （历史 bug：时长曾存组件本地 ref，切换导航组件重挂即清零重计）
function tickUptime() {
  const startedAt = appStore.frpcStartedAt;
  if (startedAt && appStore.isRunning) {
    uptimeSeconds.value = Math.max(0, Math.floor((Date.now() - startedAt) / 1000));
  } else {
    uptimeSeconds.value = 0;
  }
}

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

// ==================== 实时日志（frpc 运行时日志） ====================
interface LiveLogLine {
  ts: string;      // 格式化时间 HH:MM:SS
  line: string;    // 日志原文（含 [serverId] 前缀）
  isError: boolean;
}

const liveLogs = ref<LiveLogLine[]>([]);
const autoScroll = ref(true);
const logTerminalRef = ref<HTMLElement | null>(null);
const MAX_LOG_LINES = 500;
let unlistenLog: UnlistenFn | null = null;

function fmtTime(ms: number): string {
  const d = new Date(ms);
  const p = (n: number) => String(n).padStart(2, '0');
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

function appendLiveLog(line: string, timestamp: number) {
  liveLogs.value.push({
    ts: fmtTime(timestamp),
    line,
    isError: /ERR|error|失败|failed/i.test(line),
  });
  // 环形上限，防止长时间运行内存膨胀
  if (liveLogs.value.length > MAX_LOG_LINES) {
    liveLogs.value.splice(0, liveLogs.value.length - MAX_LOG_LINES);
  }
  // 自动滚动到底部（用户可手动上滚关闭自动跟随）
  if (autoScroll.value) {
    nextTick(() => {
      const el = logTerminalRef.value;
      if (el) el.scrollTop = el.scrollHeight;
    });
  }
}

function onTerminalScroll() {
  const el = logTerminalRef.value;
  if (!el) return;
  // 距底部 40px 内视为跟随；上滚则暂停自动滚动
  autoScroll.value = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
}

function clearLiveLogs() {
  liveLogs.value = [];
}

onMounted(async () => {
  // 立即校准一次（切页回来时恢复正确时长，而非从 0 重计）
  tickUptime();
  uptimeTimer = setInterval(tickUptime, 1000);

  // 订阅后端 frpc 实时日志事件
  try {
    unlistenLog = await listen<{ line: string; timestamp: number }>('frpc-log', (event) => {
      appendLiveLog(event.payload.line, event.payload.timestamp);
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

    <!-- frpc 运行时实时日志终端 -->
    <a-card class="logs-card">
      <template #title>
        <span>实时日志</span>
        <a-tag v-if="!autoScroll" color="orange" style="margin-left: 8px">已暂停跟随</a-tag>
      </template>
      <template #extra>
        <a-space>
          <a-checkbox v-model:checked="autoScroll" size="small">自动滚动</a-checkbox>
          <a-button size="small" @click="clearLiveLogs">清空</a-button>
          <a-button type="link" size="small" @click="router.push('/logs')">查看全部</a-button>
        </a-space>
      </template>
      <div
        ref="logTerminalRef"
        class="log-terminal"
        @scroll="onTerminalScroll"
      >
        <div v-if="liveLogs.length === 0" class="log-empty">
          暂无运行日志 —— 启动 FRP 后此处将实时显示 frpc 输出
        </div>
        <div
          v-for="(log, i) in liveLogs"
          :key="i"
          class="log-line"
          :class="{ 'log-error': log.isError }"
        >
          <span class="log-ts">{{ log.ts }}</span>
          <span class="log-text">{{ log.line }}</span>
        </div>
      </div>
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

  // 终端风格实时日志
  .log-terminal {
    background: #0f172a;
    border-radius: 8px;
    padding: 12px 16px;
    height: 360px;
    overflow-y: auto;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 12.5px;
    line-height: 1.7;

    .log-empty {
      color: #475569;
      text-align: center;
      padding-top: 150px;
      user-select: none;
    }

    .log-line {
      white-space: pre-wrap;
      word-break: break-all;
      color: #cbd5e1;

      .log-ts {
        color: #64748b;
        margin-right: 10px;
        user-select: none;
      }

      &.log-error {
        color: #f87171;

        .log-ts { color: #b91c1c; }
      }

      &:hover {
        background: rgba(255, 255, 255, 0.04);
      }
    }

    &::-webkit-scrollbar {
      width: 8px;
    }
    &::-webkit-scrollbar-thumb {
      background: #334155;
      border-radius: 4px;
    }
    &::-webkit-scrollbar-track {
      background: transparent;
    }
  }
}
</style>
