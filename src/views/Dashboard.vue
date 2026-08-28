<script setup lang="ts">
/**
 * 概览页
 *
 * 运行时长：基于全局 store 的 frpcStartedAt（后端权威时间戳同步），切页不重置
 * 代理请求：从 Admin API 轮询获取（proxyStatuses + serverTraffic），5s 自动刷新
 */
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { useRouter } from 'vue-router';
import {
  PlusOutlined, PlayCircleOutlined, PauseCircleOutlined,
  FileTextOutlined, SettingOutlined, ReloadOutlined, ClockCircleOutlined,
  ArrowDownOutlined, ArrowUpOutlined,
} from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';

const appStore = useAppStore();
const router = useRouter();

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

// 代理请求信息（从 Admin API 轮询数据构建，替代旧的 frpc stdout 日志）
// 数据源：proxyStatuses（5s 轮询）+ serverTraffic（5s 轮询），不依赖 frpc stdout 管道
const proxyRequests = computed(() => {
  const sid = appStore.defaultServerId || appStore.servers[0]?.id;
  if (!sid) {
    console.warn('[Dashboard] proxyRequests: sid 为空, servers=', appStore.servers.map(s => ({id: s.id, name: s.name})));
    return [];
  }
  const allProxies = appStore.proxies;
  const matchingProxies = allProxies.filter(p => p.server_id === sid);
  // 兜底：按 sid 没结果时，显示所有有关联服务器的代理 + 无 server_id 的代理
  const candidates = matchingProxies.length > 0
    ? matchingProxies
    : allProxies.filter(p => p.server_id || true);
  console.log('[Dashboard] proxyRequests: sid=', sid, 'all=', allProxies.length, 'matching=', matchingProxies.length, 'candidates=', candidates.length);
  return candidates.map(p => {
    const key = `${sid}-${p.name}`;
    const ps = appStore.proxyStatuses.get(key);
    return {
      name: p.name,
      type: (p.type || 'tcp').toUpperCase(),
      online: ps?.state === 'online',
      trafficIn: ps?.today_traffic_in || 0,
      trafficOut: ps?.today_traffic_out || 0,
    };
  });
});

// 默认服务器总流量
const totalTraffic = computed(() => {
  const sid = appStore.defaultServerId || appStore.servers[0]?.id;
  if (!sid) return { in: 0, out: 0 };
  const t = appStore.serverTraffic.get(sid);
  return {
    in: t?.total_traffic_in || 0,
    out: t?.total_traffic_out || 0,
  };
});

// 格式化流量
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
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

onMounted(() => {
  // 立即校准一次（切页回来时恢复正确时长，而非从 0 重计）
  tickUptime();
  uptimeTimer = setInterval(tickUptime, 1000);
  // 代理请求信息由全局 5s 轮询自动刷新，无需额外订阅
});

onUnmounted(() => {
  if (uptimeTimer) clearInterval(uptimeTimer);
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

    <!-- 代理请求 -->
    <a-card class="proxy-requests-card" :bordered="false">
      <template #title>
        <div class="card-header">
          <span>代理请求</span>
          <span class="refresh-hint">每 5 秒自动刷新</span>
        </div>
      </template>
      <div class="proxy-list">
        <div v-if="proxyRequests.length === 0" class="proxy-empty">
          暂无代理 —— 添加代理并启动 FRP 后此处将显示代理请求状态
        </div>
        <div v-for="item in proxyRequests" :key="item.name" class="proxy-row">
          <span class="proxy-dot" :class="{ 'dot-online': item.online, 'dot-offline': !item.online }"></span>
          <span class="proxy-name">{{ item.name }}</span>
          <span class="proxy-type">{{ item.type }}</span>
          <span class="proxy-state" :class="{ 'state-online': item.online, 'state-offline': !item.online }">
            {{ item.online ? 'online' : 'offline' }}
          </span>
          <span class="proxy-traffic">
            <ArrowDownOutlined /> {{ formatBytes(item.trafficIn) }}
            <ArrowUpOutlined /> {{ formatBytes(item.trafficOut) }}
          </span>
        </div>
      </div>
      <div v-if="proxyRequests.length > 0" class="traffic-summary">
        <span>总流量：</span>
        <span class="traffic-item"><ArrowDownOutlined /> {{ formatBytes(totalTraffic.in) }}</span>
        <span class="traffic-item"><ArrowUpOutlined /> {{ formatBytes(totalTraffic.out) }}</span>
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
.quick-actions-card, .proxy-requests-card { border-radius: 8px; margin-bottom: 16px; }
.card-header { display: flex; align-items: center; justify-content: space-between; }
.refresh-hint { font-size: 12px; color: #8c8c8c; font-weight: normal; }
.proxy-list {
  background: #fafafa; border-radius: 6px; padding: 8px 12px;
  min-height: 120px; max-height: 320px; overflow-y: auto;
}
.proxy-empty { color: #8c8c8c; text-align: center; margin-top: 40px; }
.proxy-row {
  display: flex; align-items: center; gap: 12px;
  padding: 8px 0; border-bottom: 1px solid #f0f0f0;
  font-size: 13px;
}
.proxy-row:last-child { border-bottom: none; }
.proxy-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.dot-online { background: #52c41a; box-shadow: 0 0 4px rgba(82,196,26,0.5); }
.dot-offline { background: #bfbfbf; }
.proxy-name { font-weight: 500; min-width: 100px; }
.proxy-type { color: #8c8c8c; min-width: 50px; font-size: 12px; }
.proxy-state { min-width: 60px; font-size: 12px; }
.state-online { color: #52c41a; }
.state-offline { color: #8c8c8c; }
.proxy-traffic { margin-left: auto; color: #595959; display: flex; align-items: center; gap: 6px; }
.traffic-summary {
  display: flex; align-items: center; gap: 16px;
  padding: 8px 12px; border-top: 2px solid #f0f0f0;
  font-size: 13px; color: #595959; margin-top: 4px;
}
.traffic-item { display: flex; align-items: center; gap: 4px; }
</style>
