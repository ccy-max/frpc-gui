<script setup lang="ts">
// N2 统计页：流量趋势曲线 + 代理流量 Top + 连接事件时间线
// 数据源：trafficHistory / connectionHistory（后端持久化 30 天）
//
// 显示约定：
// - 无数据时不渲染 echarts 画布（否则 320px 空画布会和 a-empty 图标叠加）
// - 有数据但全为 0 时显示"累计数据为 0"提示，而不是画一条贴底直线
// - 单位换算自适应（B/KB/MB/GB），避免小流量被显示成 0.00 MB
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { LineChart, BarChart } from 'echarts/charts';
import {
  GridComponent, TooltipComponent, LegendComponent, TitleComponent,
} from 'echarts/components';
import { use } from 'echarts/core';
import * as echarts from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import {
  ClockCircleOutlined, RiseOutlined, FallOutlined, ReloadOutlined,
  LineChartOutlined, BarChartOutlined,
} from '@ant-design/icons-vue';

use([LineChart, BarChart, GridComponent, TooltipComponent, LegendComponent, TitleComponent, CanvasRenderer]);

const appStore = useAppStore();

const trendRef = ref<HTMLElement>();
const topRef = ref<HTMLElement>();
let trendChart: echarts.ECharts | null = null;
let topChart: echarts.ECharts | null = null;
let unwatchHistory: (() => void) | null = null;

// ===== 单位自适应 =====
// 统计页流量通常很小（KB 级），固定用 MB 会全显示成 0.00
function pickUnit(maxBytes: number): { unit: string; divisor: number } {
  if (maxBytes >= 1024 * 1024 * 1024) return { unit: 'GB', divisor: 1024 * 1024 * 1024 };
  if (maxBytes >= 1024 * 1024) return { unit: 'MB', divisor: 1024 * 1024 };
  if (maxBytes >= 1024) return { unit: 'KB', divisor: 1024 };
  return { unit: 'B', divisor: 1 };
}
function formatBytes(bytes: number): string {
  if (!bytes) return '0 B';
  const { unit, divisor } = pickUnit(bytes);
  const v = bytes / divisor;
  return `${v >= 100 ? v.toFixed(0) : v.toFixed(2)} ${unit}`;
}

const axisColor = computed(() => (appStore.isDarkMode ? '#94a3b8' : '#64748b'));
const splitColor = computed(() => (appStore.isDarkMode ? '#334155' : '#e2e8f0'));
const textColor = computed(() => (appStore.isDarkMode ? '#e2e8f0' : '#1e293b'));
const subduedColor = computed(() => (appStore.isDarkMode ? '#64748b' : '#94a3b8'));

// ===== 派生数据 =====
const historyDays = computed(() => appStore.trafficHistory.length);

// 趋势最大值（用于选单位，同时判断是否全零）
const trendMaxBytes = computed(() => {
  let m = 0;
  for (const h of appStore.trafficHistory) {
    m = Math.max(m, h.total_traffic_in || 0, h.total_traffic_out || 0);
  }
  return m;
});

// 代理 Top10 累计（按 入+出 排序）
const topProxies = computed(() => {
  const acc = new Map<string, number>();
  for (const day of appStore.trafficHistory) {
    for (const p of (day.proxies || [])) {
      acc.set(p.name, (acc.get(p.name) || 0) + (p.traffic_in || 0) + (p.traffic_out || 0));
    }
  }
  return [...acc.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10);
});

const topMaxBytes = computed(() => topProxies.value[0]?.[1] ?? 0);

// 汇总卡片数据
const summary = computed(() => {
  let inSum = 0, outSum = 0;
  for (const h of appStore.trafficHistory) {
    inSum += h.total_traffic_in || 0;
    outSum += h.total_traffic_out || 0;
  }
  return {
    days: historyDays.value,
    inSum,
    outSum,
    total: inSum + outSum,
  };
});

// 连接事件（展平取最近 50 条）
const recentEvents = computed(() => {
  const flat: { ts: number; type: string; proxy: string; server: string; msg?: string }[] = [];
  for (const h of appStore.connectionHistory) {
    for (const e of (h.events || [])) {
      flat.push({ ts: e.timestamp, type: e.event_type, proxy: h.proxy_name, server: h.server_id, msg: e.message ?? undefined });
    }
  }
  return flat.sort((a, b) => b.ts - a.ts).slice(0, 50);
});

// 事件统计
const eventStats = computed(() => {
  let connected = 0, disconnected = 0;
  for (const e of recentEvents.value) {
    if (e.type === 'connected') connected++;
    else if (e.type === 'disconnected') disconnected++;
  }
  return { connected, disconnected };
});

// ===== 渲染 =====
function renderTrend() {
  // 无数据：销毁画布，交给 a-empty 显示（避免空画布与空状态图标叠加）
  if (!historyDays.value || !trendRef.value) {
    trendChart?.dispose();
    trendChart = null;
    return;
  }
  // init 时容器必须已有尺寸；v-show 隐藏态 clientWidth=0 会导致图表渲染为空白
  if (!trendRef.value.clientWidth) return;
  if (!trendChart) trendChart = echarts.init(trendRef.value);

  const hist = appStore.trafficHistory;
  const dates = hist.map(h => (h.date || '').slice(5));
  const { unit, divisor } = pickUnit(trendMaxBytes.value);
  const inData = hist.map(h => +((h.total_traffic_in || 0) / divisor).toFixed(2));
  const outData = hist.map(h => +((h.total_traffic_out || 0) / divisor).toFixed(2));

  trendChart.setOption({
    backgroundColor: 'transparent',
    tooltip: {
      trigger: 'axis',
      valueFormatter: (v: number) => `${v} ${unit}`,
    },
    legend: {
      data: ['入站', '出站'],
      textStyle: { color: textColor.value },
      top: 0,
      itemWidth: 14,
      itemHeight: 8,
    },
    // top 留 36 给 legend，right 留 24 给 y 轴标签
    grid: { left: 8, right: 24, top: 36, bottom: 8, containLabel: true },
    xAxis: {
      type: 'category',
      data: dates,
      boundaryGap: false,
      axisLabel: { color: axisColor.value, fontSize: 11 },
      axisLine: { lineStyle: { color: splitColor.value } },
      axisTick: { show: false },
    },
    yAxis: {
      type: 'value',
      name: unit,
      nameTextStyle: { color: subduedColor.value, fontSize: 11, align: 'left' },
      nameGap: 12,
      axisLabel: { color: axisColor.value, fontSize: 11 },
      splitLine: { lineStyle: { color: splitColor.value, type: 'dashed' } },
      axisLine: { show: false },
      axisTick: { show: false },
    },
    series: [
      {
        name: '入站', type: 'line', smooth: true, data: inData,
        symbol: 'circle', symbolSize: 5, showSymbol: dates.length <= 10,
        itemStyle: { color: '#2563eb' },
        lineStyle: { width: 2 },
        areaStyle: { color: 'rgba(37, 99, 235, 0.12)' },
      },
      {
        name: '出站', type: 'line', smooth: true, data: outData,
        symbol: 'circle', symbolSize: 5, showSymbol: dates.length <= 10,
        itemStyle: { color: '#7c3aed' },
        lineStyle: { width: 2 },
        areaStyle: { color: 'rgba(124, 58, 237, 0.12)' },
      },
    ],
  }, true); // notMerge：切换数据规模时避免残留旧系列
}

function renderTop() {
  // 无代理数据：销毁画布
  if (!topProxies.value.length || !topRef.value) {
    topChart?.dispose();
    topChart = null;
    return;
  }
  if (!topRef.value.clientWidth) return;
  if (!topChart) topChart = echarts.init(topRef.value);

  const { unit, divisor } = pickUnit(topMaxBytes.value);
  // 横向条形图：类目自下而上，reverse 让最大值显示在顶部
  const items = [...topProxies.value].reverse();
  const names = items.map(i => i[0]);
  const values = items.map(i => +((i[1] || 0) / divisor).toFixed(2));

  topChart.setOption({
    backgroundColor: 'transparent',
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      valueFormatter: (v: number) => `${v} ${unit}`,
    },
    grid: { left: 8, right: 16, top: 20, bottom: 8, containLabel: true },
    xAxis: {
      type: 'value',
      name: unit,
      nameTextStyle: { color: subduedColor.value, fontSize: 11 },
      nameGap: 8,
      axisLabel: { color: axisColor.value, fontSize: 11 },
      splitLine: { lineStyle: { color: splitColor.value, type: 'dashed' } },
      axisLine: { show: false },
      axisTick: { show: false },
    },
    yAxis: {
      type: 'category',
      data: names,
      axisLabel: {
        color: axisColor.value,
        fontSize: 11,
        width: 88,
        overflow: 'truncate',
      },
      axisLine: { lineStyle: { color: splitColor.value } },
      axisTick: { show: false },
    },
    series: [{
      type: 'bar',
      data: values,
      itemStyle: {
        color: 'rgba(59, 130, 246, 0.85)',
        borderRadius: [0, 4, 4, 0],
      },
      barMaxWidth: 18,
      label: {
        show: true,
        position: 'right',
        color: subduedColor.value,
        fontSize: 10,
        formatter: (p: any) => `${p.value}`,
      },
    }],
  }, true);
}

function renderAll() {
  renderTrend();
  renderTop();
}
function onResize() {
  trendChart?.resize();
  topChart?.resize();
}

// 手动刷新
const refreshing = ref(false);
async function manualRefresh() {
  refreshing.value = true;
  try {
    await appStore.loadTrafficHistory();
    await appStore.loadConnectionHistory();
    await nextTick();
    renderAll();
  } finally {
    refreshing.value = false;
  }
}

onMounted(async () => {
  // 订阅历史数据：5s 轮询时自动刷新（此前只在 init 加载一次，页面永远空白）
  unwatchHistory = appStore.watchTrafficHistory();
  await nextTick();
  renderAll();
  window.addEventListener('resize', onResize);
});

onUnmounted(() => {
  window.removeEventListener('resize', onResize);
  unwatchHistory?.();
  trendChart?.dispose();
  topChart?.dispose();
  trendChart = null;
  topChart = null;
});

// 数据变化/主题切换重绘：用长度 + 汇总值双重触发，
// 同一天流量增长时长度不变但数值变了，仅靠 length 不会重绘
watch(
  () => [historyDays.value, summary.value.total, appStore.isDarkMode, topProxies.value.length],
  async () => {
    await nextTick();
    renderAll();
  },
);

function eventColor(type: string): string {
  return type === 'connected' ? 'green' : type === 'disconnected' ? 'red' : 'orange';
}
function eventLabel(type: string): string {
  return type === 'connected' ? '上线' : type === 'disconnected' ? '掉线' : '错误';
}
function serverName(id: string): string {
  return appStore.servers.find(s => s.id === id)?.name || id;
}
function fmtTime(ts: number): string {
  const d = new Date(ts * 1000);
  return `${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`;
}
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <div>
        <h1 class="page-title">📊 统计分析</h1>
        <p class="page-subtitle">流量趋势、代理排行与连接事件（进程级监控，数据每 5 秒自动刷新）</p>
      </div>
      <a-button :loading="refreshing" @click="manualRefresh">
        <template #icon><ReloadOutlined /></template>
        刷新
      </a-button>
    </div>

    <!-- 汇总指标 -->
    <a-row :gutter="16" class="summary-row">
      <a-col :xs="12" :sm="12" :md="6">
        <a-card :bordered="false" class="metric-card">
          <div class="metric-label">记录天数</div>
          <div class="metric-value">
            {{ summary.days }} <span class="metric-unit">天</span>
          </div>
        </a-card>
      </a-col>
      <a-col :xs="12" :sm="12" :md="6">
        <a-card :bordered="false" class="metric-card">
          <div class="metric-label">
            <FallOutlined style="color: #2563eb; margin-right: 4px;" />入站累计
          </div>
          <div class="metric-value" style="color: #2563eb">{{ formatBytes(summary.inSum) }}</div>
        </a-card>
      </a-col>
      <a-col :xs="12" :sm="12" :md="6">
        <a-card :bordered="false" class="metric-card">
          <div class="metric-label">
            <RiseOutlined style="color: #7c3aed; margin-right: 4px;" />出站累计
          </div>
          <div class="metric-value" style="color: #7c3aed">{{ formatBytes(summary.outSum) }}</div>
        </a-card>
      </a-col>
      <a-col :xs="12" :sm="12" :md="6">
        <a-card :bordered="false" class="metric-card">
          <div class="metric-label">
            <ClockCircleOutlined style="margin-right: 4px;" />连接事件
          </div>
          <div class="metric-value" style="font-size: 18px">
            <span style="color: #22c55e">{{ eventStats.connected }}</span>
            <span class="metric-sep">/</span>
            <span style="color: #ef4444">{{ eventStats.disconnected }}</span>
          </div>
          <div class="metric-hint">上线 / 掉线</div>
        </a-card>
      </a-col>
    </a-row>

    <a-row :gutter="16">
      <!-- 流量趋势 -->
      <a-col :xs="24" :lg="16">
        <a-card class="chart-card" :bordered="false">
          <template #title>
            <span>流量趋势</span>
            <a-tag v-if="historyDays" color="blue" style="margin-left: 8px">近 {{ historyDays }} 天</a-tag>
          </template>
          <template #extra>
            <a-tooltip title="通过监控 frpc 进程 I/O 获取（GetProcessIoCounters），为进程级估算值">
              <span class="hint-link">数据说明</span>
            </a-tooltip>
          </template>

          <!-- 有数据才渲染画布，避免空画布与空状态叠加 -->
          <div v-show="historyDays" ref="trendRef" class="chart-box"></div>
          <div v-if="historyDays && trendMaxBytes === 0" class="chart-overlay-hint">
            已记录 {{ historyDays }} 天，但流量累计为 0
            <div class="hint-sub">启动 FRP 并产生实际流量后，此处会显示曲线</div>
          </div>
          <a-empty
            v-if="!historyDays"
            class="chart-empty"
            description="暂无流量历史数据"
          >
            <template #image>
              <LineChartOutlined class="empty-icon" />
            </template>
            <div class="empty-hint">
              启动 FRP 并产生流量后自动记录，<br />
              数据每 5 秒刷新一次
            </div>
          </a-empty>
        </a-card>
      </a-col>

      <!-- 代理流量 Top 10 -->
      <a-col :xs="24" :lg="8">
        <a-card class="chart-card" :bordered="false">
          <template #title>
            <span>代理流量 Top 10</span>
          </template>
          <template #extra>
            <a-tag v-if="topProxies.length" color="purple">{{ topProxies.length }} 个代理</a-tag>
          </template>

          <div v-show="topProxies.length" ref="topRef" class="chart-box"></div>
          <div v-if="topProxies.length && topMaxBytes === 0" class="chart-overlay-hint">
            代理流量均为 0
            <div class="hint-sub">代理有数据传输后才会产生排行</div>
          </div>
          <a-empty
            v-if="!topProxies.length"
            class="chart-empty"
            description="暂无代理流量数据"
          >
            <template #image>
              <BarChartOutlined class="empty-icon" />
            </template>
            <div class="empty-hint">代理产生流量后<br />此处显示 Top 10 排行</div>
          </a-empty>
        </a-card>
      </a-col>
    </a-row>

    <!-- 连接事件时间线 -->
    <a-card class="chart-card mt-4" :bordered="false">
      <template #title>
        <span>连接事件时间线</span>
        <a-tag v-if="recentEvents.length" style="margin-left: 8px">{{ recentEvents.length }} 条</a-tag>
      </template>
      <template #extra>
        <span class="hint-link">代理上线 / 掉线时记录</span>
      </template>

      <a-timeline v-if="recentEvents.length" class="event-timeline">
        <a-timeline-item
          v-for="(e, i) in recentEvents"
          :key="i"
          :color="eventColor(e.type)"
        >
          <div class="event-line">
            <a-tag :color="eventColor(e.type)" class="event-tag">{{ eventLabel(e.type) }}</a-tag>
            <span class="event-proxy">{{ e.proxy }}</span>
            <span class="event-server">@ {{ serverName(e.server) }}</span>
            <span class="event-time">{{ fmtTime(e.ts) }}</span>
            <span v-if="e.msg" class="event-msg">{{ e.msg }}</span>
          </div>
        </a-timeline-item>
      </a-timeline>
      <a-empty v-else description="暂无连接事件">
        <template #image>
          <ClockCircleOutlined class="empty-icon" />
        </template>
        <div class="empty-hint">代理上线或掉线时自动记录</div>
      </a-empty>
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.page-container { padding: 24px; }

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;

  .page-title {
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
    margin: 0 0 4px;
  }
  .page-subtitle {
    font-size: 13px;
    color: var(--text-tertiary, #94a3b8);
    margin: 0;
  }
}

/* 汇总指标 */
.summary-row {
  margin-bottom: 16px;
  :deep(.ant-col) { margin-bottom: 16px; }
}
.metric-card {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
  height: 100%;

  :deep(.ant-card-body) { padding: 16px 18px; }

  .metric-label {
    font-size: 12px;
    color: var(--text-tertiary, #94a3b8);
    margin-bottom: 6px;
    display: flex;
    align-items: center;
  }
  .metric-value {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
    line-height: 1.2;
  }
  .metric-unit {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-tertiary, #94a3b8);
  }
  .metric-sep {
    color: var(--text-disabled, #cbd5e1);
    margin: 0 4px;
    font-weight: 400;
  }
  .metric-hint {
    font-size: 11px;
    color: var(--text-disabled, #cbd5e1);
    margin-top: 2px;
  }
}

.chart-card {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
  margin-bottom: 16px;
  height: 100%;

  :deep(.ant-card-head) {
    min-height: 48px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.04);
  }
}

/* 画布固定高度：无数据时因 v-show 隐藏，不会占位 */
.chart-box {
  width: 100%;
  height: 320px;
}

/* 有数据但全零时的叠加提示 */
.chart-overlay-hint {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  font-size: 13px;
  color: var(--text-tertiary, #94a3b8);
  pointer-events: none;

  .hint-sub {
    font-size: 12px;
    color: var(--text-disabled, #cbd5e1);
    margin-top: 4px;
  }
}
/* 让 overlay 相对卡片定位 */
.chart-card :deep(.ant-card-body) { position: relative; }

/* 空状态：占据与画布相同高度，居中显示，不与坐标轴重叠 */
.chart-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 320px;
  padding: 0;

  :deep(.ant-empty-image) {
    height: auto;
    margin-bottom: 12px;
  }
  .empty-icon {
    font-size: 44px;
    color: var(--text-disabled, #cbd5e1);
  }
  .empty-hint {
    font-size: 12px;
    color: var(--text-disabled, #cbd5e1);
    line-height: 1.6;
  }
}

.hint-link {
  font-size: 12px;
  color: var(--text-tertiary, #94a3b8);
  cursor: help;
  border-bottom: 1px dashed currentColor;
}

/* 时间线 */
.event-timeline {
  padding-top: 4px;
  max-height: 420px;
  overflow-y: auto;
  padding-right: 4px;
}
.event-line {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;

  .event-tag { margin: 0; }
  .event-proxy {
    font-weight: 600;
    color: var(--text-primary, #1e293b);
  }
  .event-server {
    color: var(--text-tertiary, #94a3b8);
    font-size: 13px;
  }
  .event-time {
    color: var(--text-disabled, #cbd5e1);
    font-size: 12px;
    font-family: ui-monospace, monospace;
  }
  .event-msg {
    color: var(--text-secondary, #64748b);
    font-size: 13px;
  }
}
.mt-4 { margin-top: 16px; }

/* 深色模式微调 */
html.dark {
  .metric-card,
  .chart-card {
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }
}
</style>
