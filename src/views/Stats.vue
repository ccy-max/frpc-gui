<script setup lang="ts">
// N2 统计页：流量趋势曲线 + 代理流量 Top + 连接事件时间线
// 数据源：trafficHistory / connectionHistory（后端已持久化 30 天，此前无 UI）
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import {
  LineChart, BarChart,
} from 'echarts/charts';
import {
  GridComponent, TooltipComponent, LegendComponent, TitleComponent,
} from 'echarts/components';
import { use } from 'echarts/core';
import * as echarts from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';

use([LineChart, BarChart, GridComponent, TooltipComponent, LegendComponent, TitleComponent, CanvasRenderer]);

const appStore = useAppStore();

const trendRef = ref<HTMLElement>();
const topRef = ref<HTMLElement>();
let trendChart: echarts.ECharts | null = null;
let topChart: echarts.ECharts | null = null;

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

const axisColor = computed(() => (appStore.isDarkMode ? '#94a3b8' : '#64748b'));
const splitColor = computed(() => (appStore.isDarkMode ? '#334155' : '#e2e8f0'));
const textColor = computed(() => (appStore.isDarkMode ? '#e2e8f0' : '#1e293b'));

// 流量趋势：近 30 天入/出双折线
function renderTrend() {
  if (!trendRef.value) return;
  if (!trendChart) trendChart = echarts.init(trendRef.value);
  const hist = appStore.trafficHistory;
  const dates = hist.map(h => h.date.slice(5)); // MM-DD
  const inData = hist.map(h => +(h.total_traffic_in / 1024 / 1024).toFixed(2));
  const outData = hist.map(h => +(h.total_traffic_out / 1024 / 1024).toFixed(2));
  trendChart.setOption({
    backgroundColor: 'transparent',
    tooltip: { trigger: 'axis', valueFormatter: (v: number) => `${v} MB` },
    legend: { data: ['入站', '出站'], textStyle: { color: textColor.value } },
    grid: { left: 50, right: 20, top: 40, bottom: 30 },
    xAxis: {
      type: 'category', data: dates,
      axisLabel: { color: axisColor.value }, axisLine: { lineStyle: { color: splitColor.value } },
    },
    yAxis: {
      type: 'value', name: 'MB',
      axisLabel: { color: axisColor.value }, splitLine: { lineStyle: { color: splitColor.value } },
    },
    series: [
      { name: '入站', type: 'line', smooth: true, data: inData, itemStyle: { color: '#2563eb' }, areaStyle: { opacity: 0.12 } },
      { name: '出站', type: 'line', smooth: true, data: outData, itemStyle: { color: '#7c3aed' }, areaStyle: { opacity: 0.12 } },
    ],
  });
}

// 代理流量 Top10（30 天累计）
function renderTop() {
  if (!topRef.value) return;
  if (!topChart) topChart = echarts.init(topRef.value);
  const acc = new Map<string, number>();
  for (const day of appStore.trafficHistory) {
    for (const p of (day.proxies || [])) {
      acc.set(p.name, (acc.get(p.name) || 0) + (p.traffic_in || 0) + (p.traffic_out || 0));
    }
  }
  const sorted = [...acc.entries()].sort((a, b) => b[1] - a[1]).slice(0, 10).reverse();
  topChart.setOption({
    backgroundColor: 'transparent',
    tooltip: { trigger: 'axis', valueFormatter: (v: number) => `${v} MB` },
    grid: { left: 120, right: 30, top: 20, bottom: 30 },
    xAxis: {
      type: 'value', name: 'MB',
      axisLabel: { color: axisColor.value }, splitLine: { lineStyle: { color: splitColor.value } },
    },
    yAxis: {
      type: 'category', data: sorted.map(s => s[0]),
      axisLabel: { color: axisColor.value }, axisLine: { lineStyle: { color: splitColor.value } },
    },
    series: [{
      type: 'bar', data: sorted.map(s => +(s[1] / 1024 / 1024).toFixed(2)),
      itemStyle: { color: '#3b82f6', borderRadius: [0, 4, 4, 0] }, barWidth: '55%',
    }],
  });
}

// 连接事件时间线（展平排序取最近 50 条）
const recentEvents = computed(() => {
  const flat: { ts: number; type: string; proxy: string; server: string; msg?: string }[] = [];
  for (const h of appStore.connectionHistory) {
    for (const e of (h.events || [])) {
      flat.push({ ts: e.timestamp, type: e.event_type, proxy: h.proxy_name, server: h.server_id, msg: e.message ?? undefined });
    }
  }
  return flat.sort((a, b) => b.ts - a.ts).slice(0, 50);
});

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
  return new Date(ts * 1000).toLocaleString();
}

function renderAll() { renderTrend(); renderTop(); }
function onResize() { trendChart?.resize(); topChart?.resize(); }

onMounted(async () => {
  await nextTick();
  renderAll();
  window.addEventListener('resize', onResize);
});
onUnmounted(() => {
  window.removeEventListener('resize', onResize);
  trendChart?.dispose();
  topChart?.dispose();
});
// 数据/主题变化重绘
watch(() => [appStore.trafficHistory.length, appStore.isDarkMode], () => renderAll());
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">📊 统计分析</h1>
      <a-button @click="renderAll">
        刷新
      </a-button>
    </div>

    <a-row :gutter="16">
      <a-col :xs="24" :lg="16">
        <a-card title="流量趋势（近 30 天）" class="chart-card">
          <div ref="trendRef" class="chart-box" style="height: 320px;"></div>
          <a-empty v-if="!appStore.trafficHistory.length" description="暂无流量历史数据（启动 FRP 运行后自动记录）" />
        </a-card>
      </a-col>
      <a-col :xs="24" :lg="8">
        <a-card title="代理流量 Top 10" class="chart-card">
          <div ref="topRef" class="chart-box" style="height: 320px;"></div>
          <a-empty v-if="!recentEvents.length && !appStore.trafficHistory.length" description="暂无数据" />
        </a-card>
      </a-col>
    </a-row>

    <a-card title="连接事件时间线" class="chart-card mt-4">
      <a-timeline v-if="recentEvents.length">
        <a-timeline-item
          v-for="(e, i) in recentEvents"
          :key="i"
          :color="eventColor(e.type)"
        >
          <div class="event-line">
            <a-tag :color="eventColor(e.type)">{{ eventLabel(e.type) }}</a-tag>
            <span class="event-proxy">{{ e.proxy }}</span>
            <span class="event-server">@ {{ serverName(e.server) }}</span>
            <span class="event-time">{{ fmtTime(e.ts) }}</span>
            <span v-if="e.msg" class="event-msg">{{ e.msg }}</span>
          </div>
        </a-timeline-item>
      </a-timeline>
      <a-empty v-else description="暂无连接事件（代理上线/掉线时记录）" />
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.page-container { padding: 24px; }
.page-header {
  display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px;
  .page-title { font-size: 24px; font-weight: 700; color: var(--text-primary); margin: 0; }
}
.chart-card {
  border-radius: var(--radius-lg);
  margin-bottom: 16px;
}
.chart-box { width: 100%; }
.event-line {
  display: flex; align-items: center; gap: 10px; flex-wrap: wrap;
  .event-proxy { font-weight: 600; color: var(--text-primary); }
  .event-server { color: var(--text-tertiary); font-size: 13px; }
  .event-time { color: var(--text-disabled); font-size: 12px; font-family: monospace; }
  .event-msg { color: var(--text-secondary); font-size: 13px; }
}
</style>
