<script setup lang="ts">
/**
 * N7 HTTP 请求检查器（对标 ngrok Traffic Inspector）
 *
 * 用法：本地服务跑在 8080 → 检查器监听 18080 反代到 127.0.0.1:8080
 *      → 把 frpc http 代理的 local_port 改成 18080 → 所有请求被捕获
 * 能力：实时请求列表、详情（头/体）、一键重放、清空
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { message } from 'ant-design-vue';
import { ReloadOutlined, ClearOutlined, ThunderboltOutlined } from '@ant-design/icons-vue';

interface Req {
  id: number;
  timestamp: number;
  method: string;
  path: string;
  req_headers: [string, string][];
  req_body: string;
  status: number;
  resp_headers: [string, string][];
  resp_body: string;
  duration_ms: number;
  replayed: boolean;
  error: string | null;
}

const running = ref(false);
const port = ref(18080);
const upstream = ref('127.0.0.1:8080');
const actualPort = ref<number | null>(null);
const requests = ref<Req[]>([]);
const selected = ref<Req | null>(null);
const detailOpen = ref(false);
const loading = ref(false);
let pollTimer: ReturnType<typeof setInterval> | null = null;

const reversed = computed(() => [...requests.value].reverse());

function statusColor(s: number): string {
  if (s >= 500) return 'red';
  if (s >= 400) return 'orange';
  if (s >= 300) return 'blue';
  if (s >= 200) return 'green';
  return 'default';
}
function methodColor(m: string): string {
  const c: Record<string, string> = { GET: 'blue', POST: 'green', PUT: 'orange', DELETE: 'red', PATCH: 'purple' };
  return c[m] || 'default';
}
function fmtTime(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString();
}

async function refresh() {
  try {
    const [r, p, up] = await invoke<[boolean, number | null, string | null]>('inspector_status');
    running.value = r;
    actualPort.value = p;
    if (r && up) upstream.value = up;
    if (r) requests.value = await invoke<Req[]>('inspector_list');
  } catch (e) {
    console.error('[inspector] status failed:', e);
  }
}

async function start() {
  loading.value = true;
  try {
    await invoke('inspector_start', { port: port.value, upstream: upstream.value });
    message.success(`检查器已启动：127.0.0.1:${port.value} → ${upstream.value}`);
    await refresh();
  } catch (e) {
    message.error(`启动失败：${typeof e === 'string' ? e : (e as Error).message}`);
  } finally {
    loading.value = false;
  }
}

async function stop() {
  try {
    await invoke('inspector_stop');
    message.info('检查器已停止');
    await refresh();
  } catch (e) {
    message.error(`停止失败：${String(e)}`);
  }
}

async function clear() {
  await invoke('inspector_clear');
  requests.value = [];
  message.success('已清空');
}

async function replay(req: Req) {
  try {
    await invoke('inspector_replay', { id: req.id });
    message.success(`已重放 ${req.method} ${req.path}`);
    await refresh();
  } catch (e) {
    message.error(`重放失败：${typeof e === 'string' ? e : (e as Error).message}`);
  }
}

function openDetail(req: Req) {
  selected.value = req;
  detailOpen.value = true;
}

onMounted(() => {
  refresh();
  pollTimer = setInterval(refresh, 2000);
});
onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title"><ThunderboltOutlined class="header-icon" /> 请求检查器</h1>
      <a-space>
        <a-button @click="refresh"><template #icon><ReloadOutlined /></template>刷新</a-button>
        <a-button danger :disabled="!requests.length" @click="clear"><template #icon><ClearOutlined /></template>清空</a-button>
      </a-space>
    </div>

    <!-- 控制区 -->
    <a-card class="ctrl-card" :bordered="false">
      <a-row :gutter="16" align="middle">
        <a-col :xs="24" :md="6">
          <a-form-item label="监听端口" style="margin-bottom: 0">
            <a-input-number v-model:value="port" :min="1024" :max="65535" :disabled="running" style="width: 100%" />
          </a-form-item>
        </a-col>
        <a-col :xs="24" :md="8">
          <a-form-item label="上游服务（本地真实服务）" style="margin-bottom: 0">
            <a-input v-model:value="upstream" :disabled="running" placeholder="127.0.0.1:8080" />
          </a-form-item>
        </a-col>
        <a-col :xs="24" :md="6">
          <a-space style="margin-top: 30px">
            <a-button v-if="!running" type="primary" :loading="loading" @click="start">启动检查器</a-button>
            <a-button v-else danger @click="stop">停止</a-button>
          </a-space>
        </a-col>
        <a-col :xs="24" :md="4">
          <div style="margin-top: 30px">
            <a-tag :color="running ? 'green' : 'default'">{{ running ? `运行中 :${actualPort}` : '未运行' }}</a-tag>
          </div>
        </a-col>
      </a-row>
      <a-alert
        style="margin-top: 16px"
        type="info"
        show-icon
        message="怎么用"
        description="例：本地 Web 服务在 8080 → 启动检查器（监听 18080，转发到 127.0.0.1:8080）→ 把 frpc HTTP 代理的本地端口改为 18080 → 所有外网请求即在此实时可见，并可一键重放。"
      />
    </a-card>

    <!-- 请求列表 -->
    <a-card :bordered="false" class="list-card">
      <template #title>捕获的请求（{{ requests.length }} / 200）</template>
      <a-table
        :data-source="reversed"
        :columns="[
          { title: '时间', key: 'time', width: 100 },
          { title: '方法', key: 'method', width: 90 },
          { title: '路径', dataIndex: 'path', key: 'path', ellipsis: true },
          { title: '状态', key: 'status', width: 80 },
          { title: '耗时', key: 'dur', width: 90 },
          { title: '操作', key: 'action', width: 150 },
        ]"
        row-key="id"
        size="small"
        :pagination="{ pageSize: 30, hideOnSinglePage: true }"
        :locale="{ emptyText: running ? '等待请求…' : '启动检查器后，经过的 HTTP 请求将实时显示在这里' }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'time'">
            <span class="mono">{{ fmtTime(record.timestamp) }}</span>
          </template>
          <template v-else-if="column.key === 'method'">
            <a-tag :color="methodColor(record.method)">{{ record.method }}</a-tag>
            <a-tag v-if="record.replayed" color="purple">重放</a-tag>
          </template>
          <template v-else-if="column.key === 'status'">
            <a-tag v-if="record.status" :color="statusColor(record.status)">{{ record.status }}</a-tag>
            <a-tag v-else color="red">ERR</a-tag>
          </template>
          <template v-else-if="column.key === 'dur'">
            <span class="mono">{{ record.duration_ms }}ms</span>
          </template>
          <template v-else-if="column.key === 'action'">
            <a-space>
              <a @click="openDetail(record)">详情</a>
              <a @click="replay(record)">重放</a>
            </a-space>
          </template>
        </template>
      </a-table>
    </a-card>

    <!-- 详情抽屉 -->
    <a-drawer v-model:open="detailOpen" :title="selected ? `${selected.method} ${selected.path}` : ''" width="640">
      <template v-if="selected">
        <a-descriptions :column="1" size="small" bordered>
          <a-descriptions-item label="状态">
            <a-tag :color="statusColor(selected.status)">{{ selected.status || 'ERROR' }}</a-tag>
            <span v-if="selected.error" style="color:#ef4444; margin-left:8px">{{ selected.error }}</span>
          </a-descriptions-item>
          <a-descriptions-item label="耗时">{{ selected.duration_ms }}ms</a-descriptions-item>
        </a-descriptions>

        <h4 class="sec">请求头</h4>
        <div class="kv-box">
          <div v-for="(h, i) in selected.req_headers" :key="i" class="kv"><b>{{ h[0] }}</b>: {{ h[1] }}</div>
        </div>

        <h4 class="sec">请求体</h4>
        <pre class="body-box">{{ selected.req_body || '(空)' }}</pre>

        <h4 class="sec">响应头</h4>
        <div class="kv-box">
          <div v-for="(h, i) in selected.resp_headers" :key="i" class="kv"><b>{{ h[0] }}</b>: {{ h[1] }}</div>
        </div>

        <h4 class="sec">响应体 <a-tag v-if="selected.resp_body.length >= 8192" color="orange">截断至 8KB</a-tag></h4>
        <pre class="body-box">{{ selected.resp_body || '(空)' }}</pre>
      </template>
    </a-drawer>
  </div>
</template>

<style scoped lang="scss">
.page-container { padding: 24px; }
.page-header {
  display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px;
  .page-title { font-size: 24px; font-weight: 700; color: var(--text-primary); margin: 0; display: flex; align-items: center; gap: 12px;
    .header-icon { font-size: 26px; color: #7c3aed; } }
}
.ctrl-card, .list-card { border-radius: 12px; margin-bottom: 16px; }
.mono { font-family: 'Consolas', monospace; font-size: 12px; }
.sec { margin: 20px 0 8px; color: var(--text-secondary); }
.kv-box {
  background: var(--bg-subtle); border: 1px solid var(--border-color); border-radius: 8px;
  padding: 10px 14px; max-height: 200px; overflow-y: auto;
  .kv { font-family: monospace; font-size: 12px; padding: 2px 0; color: var(--text-primary); word-break: break-all; }
}
.body-box {
  background: #1e293b; color: #e2e8f0; border-radius: 8px; padding: 12px 14px;
  font-family: 'Consolas', monospace; font-size: 12px; max-height: 260px; overflow: auto;
  white-space: pre-wrap; word-break: break-all;
}
</style>
