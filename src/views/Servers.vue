<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useAppStore } from '@/stores/app';
import ServerEdit from '@/views/ServerEdit.vue';
import { PlusOutlined, CloudServerOutlined, CheckCircleOutlined, PlayCircleOutlined, PauseCircleOutlined, ReloadOutlined, LoadingOutlined, CopyOutlined, EditOutlined, DeleteOutlined, StarOutlined } from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';

const appStore = useAppStore();

const searchKeyword = ref(localStorage.getItem('serverSearch') || '');
watch(searchKeyword, v => localStorage.setItem('serverSearch', v));
// 页内子页面模式（替代弹窗）：null = 列表，对象 = 编辑，'new' = 新建
const editingServer = ref<any | null>(null);
const editMode = ref(false);

const filteredServers = computed(() => {
  if (!searchKeyword.value.trim()) return appStore.servers;
  const kw = searchKeyword.value.toLowerCase();
  return appStore.servers.filter(s => 
    s.name.toLowerCase().includes(kw) || 
    s.serverAddr.toLowerCase().includes(kw)
  );
});

// 获取服务器的代理数量
function getProxyCount(serverId: string): number {
  return appStore.proxies.filter(p => p.server_id === serverId).length;
}

// 获取服务器状态
function getServerStatus(serverId: string): any {
  return appStore.serverStatuses.get(serverId);
}

// 统一格式化 Tauri invoke 错误：
// 后端 Result<T, String> 失败时 reject 的是 string 而非 Error 对象，
// 直接取 .message 会得到 undefined（历史 bug：显示"启动失败：undefined"）
function formatError(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  try { return JSON.stringify(e); } catch { return String(e); }
}

// 启动服务器
async function startServer(server: any) {
  try {
    await appStore.startServer(server.id);
    message.success(`服务器 ${server.name} 已启动`);
  } catch (e) {
    message.error(`启动失败：${formatError(e)}`);
  }
}

// 停止服务器
async function stopServer(server: any) {
  try {
    await appStore.stopServer(server.id);
    message.success(`服务器 ${server.name} 已停止`);
  } catch (e) {
    message.error(`停止失败：${formatError(e)}`);
  }
}

// 重启服务器
async function restartServer(server: any) {
  try {
    await appStore.restartServer(server.id);
    message.success(`服务器 ${server.name} 已重启`);
  } catch (e) {
    message.error(`重启失败：${formatError(e)}`);
  }
}

function openAdd() {
  editingServer.value = null;
  editMode.value = true;
}

// B1x：命令面板"新建服务器"入口
function onOpenAddServer() { openAdd(); }
onMounted(() => window.addEventListener('open-add-server', onOpenAddServer));
onUnmounted(() => window.removeEventListener('open-add-server', onOpenAddServer));

function openEdit(server: any) {
  editingServer.value = { ...server };
  editMode.value = true;
}

function closeEdit() {
  editMode.value = false;
  editingServer.value = null;
}

function onSaved() {
  closeEdit();
}

// B4x：复制服务器地址
async function copyAddr(server: any) {
  try {
    await navigator.clipboard.writeText(`${server.serverAddr}:${server.serverPort}`);
    message.success('地址已复制');
  } catch {
    message.error('复制失败');
  }
}

// F3：pending 操作显示文案
function opLabel(op: string): string {
  return op === 'starting' ? '启动中…' : op === 'stopping' ? '停止中…' : '重启中…';
}

// B3x：删除确认文案带对象名 + 关联影响
function deleteConfirmText(server: any): string {
  const n = getProxyCount(server.id);
  return n > 0
    ? `删除服务器「${server.name}」后，其下 ${n} 个代理将失去关联。`
    : `确定删除服务器「${server.name}」？`;
}

async function handleDelete(server: any) {
  // 检查是否有关联的代理
  const relatedProxies = appStore.proxies.filter(p => p.server_id === server.id);

  // 如果有关联的代理，清空它们的 server_id
  if (relatedProxies.length > 0) {
    for (const proxy of relatedProxies) {
      await appStore.updateProxy(proxy.name, { server_id: null });
    }
    message.info(`已清空 ${relatedProxies.length} 个代理的服务器关联`);
  }

  // 如果删除的是默认服务器，清空默认设置
  if (appStore.defaultServerId === server.id) {
    appStore.setDefaultServerId(null);
    message.warning('默认服务器已删除，默认设置已清空');
  }

  await appStore.deleteServer(server.id);
  message.success(`服务器「${server.name}」已删除`);
}

async function setActiveServer(server: any) {
  // 取消其他服务器的活动状态
  for (const s of appStore.servers) {
    if (s.id !== server.id) {
      await appStore.updateServer(s.id, { active: false });
    }
  }
  // 设置当前服务器为活动
  await appStore.updateServer(server.id, { active: true });
  message.success(`已切换到服务器：${server.name}`);
}
</script>

<template>
  <!-- 编辑/新建：页内子页面（替代弹窗） -->
  <ServerEdit
    v-if="editMode"
    :server="editingServer"
    @close="closeEdit"
    @saved="onSaved"
  />

  <!-- 列表视图 -->
  <div v-else class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <CloudServerOutlined class="header-icon" />
        服务器管理
      </h1>
      <a-button type="primary" @click="openAdd">
        <template #icon><PlusOutlined /></template>
        添加服务器
      </a-button>
    </div>

    <a-input
      v-model:value="searchKeyword"
      placeholder="搜索服务器名称或地址"
      class="search-input"
      allow-clear
      style="max-width: 320px; margin-bottom: 24px;"
    />

    <div v-if="filteredServers.length === 0" class="empty-state">
      <div class="empty-card">
        <div class="empty-icon">
          <CloudServerOutlined />
        </div>
        <h3 class="empty-title">
          {{ searchKeyword ? '没有匹配的服务器' : '还没有服务器配置' }}
        </h3>
        <p class="empty-desc">
          <template v-if="searchKeyword">
            换个关键词试试，或清空搜索框查看全部
          </template>
          <template v-else>
            添加 frps 服务器地址与令牌，即可开始建立内网穿透
          </template>
        </p>
        <a-button v-if="!searchKeyword" type="primary" size="large" @click="openAdd">
          <template #icon><PlusOutlined /></template>
          添加第一个服务器
        </a-button>
        <a-button v-else @click="searchKeyword = ''">清空搜索</a-button>
        <p v-if="!searchKeyword" class="empty-hint">
          配置后可设置默认服务器，简化后续代理创建
        </p>
      </div>
    </div>

    <div v-else class="server-grid">
      <div
        v-for="server in filteredServers"
        :key="server.id"
        class="server-card"
        :class="{
          'is-running': getServerStatus(server.id)?.running,
          'is-active': server.active,
        }"
      >
        <!-- 卡片头部：名称 + 状态 -->
        <div class="card-head">
          <div class="head-left">
            <span class="status-led" :class="{ on: getServerStatus(server.id)?.running }"></span>
            <span class="server-name" :title="server.name">{{ server.name }}</span>
          </div>
          <a-tag
            v-if="getServerStatus(server.id)?.running"
            color="success"
            class="head-tag"
          >运行中</a-tag>
          <a-tag v-else-if="server.active" color="processing" class="head-tag">活动中</a-tag>
          <a-tag v-else class="head-tag">已停止</a-tag>
        </div>

        <!-- 地址行（主要信息，放大显示） -->
        <div class="addr-line">
          <span class="addr-text">{{ server.serverAddr }}<span class="addr-port">:{{ server.serverPort }}</span></span>
          <CopyOutlined class="copy-icon" title="复制地址" @click="copyAddr(server)" />
        </div>

        <!-- 属性行（紧凑横排） -->
        <div class="meta-row">
          <span class="meta-chip">
            代理 <b>{{ getProxyCount(server.id) }}</b>
          </span>
          <span class="meta-chip" :class="{ dim: !server.tlsEnable }">
            TLS {{ server.tlsEnable ? '开' : '关' }}
          </span>
          <span class="meta-chip" :class="{ dim: !server.enabled }">
            {{ server.enabled ? '已启用' : '已禁用' }}
          </span>
          <span v-if="getServerStatus(server.id)?.pid" class="meta-chip mono">
            PID {{ getServerStatus(server.id)?.pid }}
          </span>
        </div>

        <!-- 操作区 -->
        <div class="card-actions">
          <template v-if="appStore.pendingOps.get(server.id)">
            <a-button size="small" disabled class="act-btn">
              <LoadingOutlined spin /> {{ opLabel(appStore.pendingOps.get(server.id)!) }}
            </a-button>
          </template>
          <template v-else-if="getServerStatus(server.id)?.running">
            <a-button size="small" danger class="act-btn" @click="stopServer(server)">
              <template #icon><PauseCircleOutlined /></template>停止
            </a-button>
            <a-button size="small" class="act-btn" @click="restartServer(server)">
              <template #icon><ReloadOutlined /></template>重启
            </a-button>
          </template>
          <template v-else>
            <a-button size="small" type="primary" class="act-btn" @click="startServer(server)">
              <template #icon><PlayCircleOutlined /></template>启动
            </a-button>
          </template>

          <span class="actions-spacer"></span>

          <a-tooltip title="编辑">
            <a-button size="small" type="text" class="icon-btn" @click="openEdit(server)">
              <template #icon><EditOutlined /></template>
            </a-button>
          </a-tooltip>
          <a-tooltip v-if="!server.active" title="设为活动服务器">
            <a-button size="small" type="text" class="icon-btn" @click="setActiveServer(server)">
              <template #icon><StarOutlined /></template>
            </a-button>
          </a-tooltip>
          <a-popconfirm
            :title="deleteConfirmText(server)"
            :ok-text="`删除「${server.name}」`"
            ok-type="danger"
            @confirm="handleDelete(server)"
          >
            <a-tooltip title="删除">
              <a-button size="small" type="text" danger class="icon-btn">
                <template #icon><DeleteOutlined /></template>
              </a-button>
            </a-tooltip>
          </a-popconfirm>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped lang="scss">
.page-container {
  padding: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;

  .page-title {
    font-size: 24px;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
    display: flex;
    align-items: center;
    gap: 12px;

    .header-icon {
      font-size: 28px;
      color: #3b82f6;
    }
  }

  .ant-btn {
    border-radius: 8px;
    font-weight: 500;
    transition: all 0.2s ease;

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 2px 8px rgba(37, 99, 235, 0.3);
    }

    &:active {
      transform: scale(0.98);
    }
  }
}

.search-input {
  border-radius: 9px;
  max-width: 300px;

  :deep(.ant-input) {
    font-size: 13px;
  }
}

.empty-state {
  margin: 40px 0;
  display: flex;
  justify-content: center;
}

.empty-card {
  width: 100%;
  max-width: 400px;
  padding: 40px 32px;
  background: #fff;
  border: 1px dashed #dbe3ee;
  border-radius: 16px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;

  .empty-icon {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 26px;
    color: #2563eb;
    background: linear-gradient(135deg, rgba(37, 99, 235, 0.12), rgba(124, 58, 237, 0.08));
    margin-bottom: 16px;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 600;
    color: #1e293b;
    margin: 0 0 6px;
  }

  .empty-desc {
    font-size: 13px;
    color: #94a3b8;
    margin: 0 0 20px;
    line-height: 1.6;
  }

  .empty-hint {
    font-size: 12px;
    color: #cbd5e1;
    margin: 14px 0 0;
  }
}

.server-grid {
  display: grid;
  // 关键修复：用固定 max 宽度而非 1fr，避免单张卡片被拉伸铺满整行
  grid-template-columns: repeat(auto-fill, minmax(300px, 360px));
  gap: 16px;
  align-items: start;
}

/* ===== 服务器卡片 ===== */
.server-card {
  background: #fff;
  border-radius: 12px;
  border: 1px solid #eef2f7;
  padding: 16px 18px 14px;
  transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease;
  display: flex;
  flex-direction: column;
  gap: 10px;

  &:hover {
    border-color: #dbeafe;
    box-shadow: 0 4px 16px rgba(15, 23, 42, 0.08);
    transform: translateY(-2px);
  }

  &.is-running {
    border-color: rgba(34, 197, 94, 0.4);
    background: linear-gradient(180deg, rgba(240, 253, 244, 0.7) 0%, #fff 46%);
  }

  /* 头部 */
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 24px;

    .head-left {
      display: flex;
      align-items: center;
      gap: 8px;
      min-width: 0;
    }

    .status-led {
      width: 7px;
      height: 7px;
      border-radius: 50%;
      flex-shrink: 0;
      background: #cbd5e1;

      &.on {
        background: #22c55e;
        box-shadow: 0 0 0 3px rgba(34, 197, 94, 0.16);
      }
    }

    .server-name {
      font-size: 15px;
      font-weight: 600;
      color: #0f172a;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .head-tag {
      margin: 0;
      flex-shrink: 0;
      font-size: 11px;
      line-height: 18px;
      padding: 0 7px;
      border-radius: 5px;
    }
  }

  /* 地址行 */
  .addr-line {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 10px;
    background: #f8fafc;
    border-radius: 8px;

    .addr-text {
      font-family: ui-monospace, 'SF Mono', Menlo, monospace;
      font-size: 13px;
      color: #334155;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      flex: 1;
      min-width: 0;
    }

    .addr-port {
      color: #94a3b8;
    }

    .copy-icon {
      color: #94a3b8;
      cursor: pointer;
      font-size: 12px;
      flex-shrink: 0;
      transition: color 0.15s;

      &:hover { color: #2563eb; }
    }
  }

  /* 属性行 */
  .meta-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;

    .meta-chip {
      font-size: 11px;
      line-height: 20px;
      padding: 0 8px;
      border-radius: 5px;
      background: #f1f5f9;
      color: #475569;
      white-space: nowrap;

      b {
        font-weight: 600;
        color: #2563eb;
      }

      &.dim {
        color: #94a3b8;
        background: #f8fafc;
      }

      &.mono {
        font-family: ui-monospace, monospace;
        color: #7c3aed;
        background: rgba(124, 58, 237, 0.07);
      }
    }
  }

  /* 操作区 */
  .card-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-top: 10px;
    border-top: 1px solid #f1f5f9;
    margin-top: 2px;

    .actions-spacer { flex: 1; }

    .act-btn {
      border-radius: 7px;
      font-size: 12px;
      height: 28px;
      padding: 0 10px;
    }

    .icon-btn {
      width: 28px;
      height: 28px;
      padding: 0;
      border-radius: 7px;
      color: #94a3b8;

      &:hover {
        color: #2563eb;
        background: rgba(37, 99, 235, 0.08);
      }

      &.ant-btn-dangerous:hover {
        color: #ef4444;
        background: rgba(239, 68, 68, 0.08);
      }
    }
  }
}

/* 深色模式 */
html.dark {
  .server-card {
    background: #1e293b;
    border-color: #334155;

    &:hover {
      border-color: #3b82f6;
      box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    }

    &.is-running {
      border-color: rgba(34, 197, 94, 0.35);
      background: linear-gradient(180deg, rgba(34, 197, 94, 0.07) 0%, #1e293b 46%);
    }

    .card-head .server-name { color: #e2e8f0; }
    .addr-line { background: #0f172a; }
    .addr-line .addr-text { color: #cbd5e1; }
    .meta-row .meta-chip { background: #0f172a; color: #94a3b8; }
    .meta-row .meta-chip.dim { color: #64748b; }
    .card-actions { border-top-color: #334155; }
  }
}

.ant-modal {
  .ant-modal-content {
    border-radius: 12px;
  }

  .ant-modal-header {
    border-radius: 12px 12px 0 0;
    font-weight: 600;
  }

  .ant-btn-primary {
    border-radius: 8px;

    &:active {
      transform: scale(0.98);
    }
  }
}
</style>
