<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { PlusOutlined, CloudServerOutlined, CheckCircleOutlined, PlayCircleOutlined, PauseCircleOutlined, ReloadOutlined, LoadingOutlined, CopyOutlined } from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';
import type { FormInstance, Rule } from 'ant-design-vue/es/form';

const appStore = useAppStore();

const searchKeyword = ref(localStorage.getItem('serverSearch') || '');
watch(searchKeyword, v => localStorage.setItem('serverSearch', v));
const modalVisible = ref(false);
const editingId = ref<string | null>(null);
const formRef = ref<FormInstance>();
const saving = ref(false);
const form = ref<any>({
  id: '', name: '', serverAddr: '', serverPort: 7000, token: '', tlsEnable: false, enabled: true,
  user: '', metaToken: '',
});

// A2 表单校验：字段级 rules，错误定位到具体输入框
const serverRules: Record<string, Rule[]> = {
  name: [
    { required: true, message: '请输入服务器名称', trigger: 'blur' },
    {
      validator: async (_r: Rule, value: string) => {
        const dup = appStore.servers.some(s => s.name === value && s.id !== editingId.value);
        if (dup) throw new Error('该名称已存在');
      },
      trigger: 'blur',
    },
  ],
  serverAddr: [
    { required: true, message: '请输入服务器地址', trigger: 'blur' },
    {
      validator: async (_r: Rule, value: string) => {
        if (!value) return;
        const ipOk = /^(\d{1,3}\.){3}\d{1,3}$/.test(value);
        const domainOk = /^[a-zA-Z0-9][-a-zA-Z0-9.]*\.[a-zA-Z]{2,}$/.test(value);
        if (!ipOk && !domainOk) throw new Error('请输入合法的 IPv4 地址或域名');
      },
      trigger: 'blur',
    },
  ],
  serverPort: [
    { required: true, type: 'number', min: 1, max: 65535, message: '端口范围 1-65535', trigger: 'blur' },
  ],
};

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
  editingId.value = null;
  form.value = { id: Date.now().toString(), name: '', serverAddr: '', serverPort: 7000, token: '', tlsEnable: false, enabled: true };
  modalVisible.value = true;
  formRef.value?.clearValidate();
}

// B1x：命令面板"新建服务器"入口
function onOpenAddServer() { openAdd(); }
onMounted(() => window.addEventListener('open-add-server', onOpenAddServer));
onUnmounted(() => window.removeEventListener('open-add-server', onOpenAddServer));

function openEdit(server: any) {
  editingId.value = server.id;
  form.value = { ...server };
  modalVisible.value = true;
  formRef.value?.clearValidate();
}

function handleSave() {
  // A2：走 a-form rules 校验，错误定位到字段
  formRef.value?.validate().then(() => {
    if (editingId.value) {
      appStore.updateServer(editingId.value, form.value);
      message.success('保存成功');
    } else {
      appStore.addServer(form.value);
      message.success('添加成功');
    }
    modalVisible.value = false;
  }).catch(() => {
    // 校验失败：字段已内联标红，无需额外提示
  });
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
  <div class="page-container">
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
      <a-empty description="暂无服务器配置">
        <template #description>
          <div class="empty-guide">
            <p style="margin-bottom: 16px; font-size: 14px; color: #64748b;">
              暂无服务器配置
            </p>
            <a-button type="primary" @click="openAdd">
              <template #icon><PlusOutlined /></template>
              添加第一个服务器
            </a-button>
            <p class="help-text" style="margin-top: 12px; font-size: 13px;">
              💡 提示：服务器用于配置 FRP 连接信息，可设置默认服务器简化代理配置
            </p>
          </div>
        </template>
      </a-empty>
    </div>

    <div v-else class="server-grid">
      <a-card
        v-for="server in filteredServers"
        :key="server.id"
        class="server-card"
        :class="{ 'server-card-active': server.active }"
        hoverable
      >
        <template #title>
          <div class="server-card-title">
            <span class="server-name">{{ server.name }}</span>
            <div class="server-tags">
              <a-tag v-if="getServerStatus(server.id)?.running" color="green">
                <template #icon><CheckCircleOutlined /></template>
                运行中
              </a-tag>
              <a-tag v-else-if="server.active" color="blue">
                <template #icon><CheckCircleOutlined /></template>
                活动中
              </a-tag>
              <a-tag v-if="getServerStatus(server.id)?.pid" color="purple" style="font-family: monospace;">
                PID: {{ getServerStatus(server.id)?.pid }}
              </a-tag>
            </div>
          </div>
        </template>

        <div class="server-card-content">
          <div class="server-info-row">
            <span class="info-label">地址：</span>
            <span class="info-value">{{ server.serverAddr }}</span>
            <CopyOutlined class="copy-icon" @click="copyAddr(server)" />
          </div>
          <div class="server-info-row">
            <span class="info-label">端口：</span>
            <span class="info-value">{{ server.serverPort }}</span>
          </div>
          <div class="server-info-row">
            <span class="info-label">代理：</span>
            <a-tag :color="getProxyCount(server.id) > 0 ? 'blue' : 'default'" size="small">
              {{ getProxyCount(server.id) }} 个
            </a-tag>
          </div>
          <div class="server-info-row">
            <span class="info-label">TLS：</span>
            <a-tag :color="server.tlsEnable ? 'green' : 'default'">
              {{ server.tlsEnable ? '启用' : '禁用' }}
            </a-tag>
          </div>
          <div class="server-info-row">
            <span class="info-label">状态：</span>
            <a-tag :color="server.enabled ? 'green' : 'default'">
              {{ server.enabled ? '启用' : '禁用' }}
            </a-tag>
          </div>
          <div v-if="server.token" class="server-info-row">
            <span class="info-label">令牌：</span>
            <span class="info-value">{{ '*'.repeat(Math.min(server.token.length, 8)) }}</span>
          </div>
        </div>

        <template #actions>
          <!-- 进程控制按钮（F3：pending 时 loading + 禁用防连点） -->
          <template v-if="appStore.pendingOps.get(server.id)">
            <a style="color: #94a3b8; cursor: not-allowed;">
              <LoadingOutlined spin /> {{ opLabel(appStore.pendingOps.get(server.id)!) }}
            </a>
          </template>
          <template v-else-if="getServerStatus(server.id)?.running">
            <a @click="stopServer(server)" style="color: #ef4444;">
              <PauseCircleOutlined /> 停止
            </a>
            <a @click="restartServer(server)">
              <ReloadOutlined /> 重启
            </a>
          </template>
          <template v-else>
            <a @click="startServer(server)" style="color: #10b981;">
              <PlayCircleOutlined /> 启动
            </a>
          </template>
          
          <a-divider type="vertical" />
          
          <!-- 常规操作 -->
          <a @click="openEdit(server)">编辑</a>
          <a @click="setActiveServer(server)" v-if="!server.active">设为活动</a>
          <a-popconfirm
            :title="deleteConfirmText(server)"
            :ok-text="`删除「${server.name}」`"
            ok-type="danger"
            @confirm="handleDelete(server)"
          >
            <a class="text-danger">删除</a>
          </a-popconfirm>
        </template>
      </a-card>
    </div>

    <a-modal
      v-model:open="modalVisible"
      :title="editingId ? '编辑服务器' : '添加服务器'"
      @ok="handleSave"
      width="600px"
    >
      <a-form ref="formRef" :model="form" :rules="serverRules" layout="vertical">
        <a-form-item label="名称" name="name">
          <a-input v-model:value="form.name" placeholder="请输入服务器名称" />
        </a-form-item>
        <a-form-item label="服务器地址" name="serverAddr">
          <a-input v-model:value="form.serverAddr" placeholder="例如：127.0.0.1 或 frp.example.com" />
        </a-form-item>
        <a-form-item label="端口" name="serverPort">
          <a-input-number v-model:value="form.serverPort" :min="1" :max="65535" style="width: 100%" />
        </a-form-item>
        <a-form-item label="令牌">
          <a-input-password v-model:value="form.token" placeholder="请输入令牌" />
        </a-form-item>
        <a-form-item label="用户名（multiuser 插件）">
          <a-input v-model:value="form.user" placeholder="可选：frps 启用 multiuser 插件时填写" />
        </a-form-item>
        <a-form-item label="meta_token（multiuser 插件）">
          <a-input-password v-model:value="form.metaToken" placeholder="可选：与 user 配套使用" />
          <div class="form-help">frps 端启用 multiuser 插件时，user + meta_token 用于多用户鉴权</div>
        </a-form-item>
        <a-form-item label="启用 TLS">
          <a-switch v-model:checked="form.tlsEnable" />
        </a-form-item>
        <a-form-item label="启用">
          <a-switch v-model:checked="form.enabled" />
        </a-form-item>
      </a-form>
    </a-modal>
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
  border-radius: 8px;
}

.empty-state {
  margin: 48px 0;
}

.server-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 24px;
}

.server-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  }

  &.server-card-active {
    border: 2px solid #52c41a;
    box-shadow: 0 4px 16px rgba(82, 196, 26, 0.3);
  }

  .server-card-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;

    .server-name {
      font-size: 16px;
      font-weight: 600;
      color: #1e293b;
    }
    
    .server-tags {
      display: flex;
      gap: 6px;
      flex-wrap: wrap;
    }
  }

  .server-card-content {
    .server-info-row {
      margin-bottom: 12px;
      display: flex;
      align-items: center;
      gap: 8px;

      .info-label {
        font-weight: 500;
        color: #64748b;
        min-width: 50px;
      }

      .info-value {
        color: #1e293b;
        font-family: 'Courier New', monospace;
      }
    }
  }

  .copy-icon {
    color: #94a3b8;
    cursor: pointer;
    font-size: 13px;
    transition: color 0.2s;
    &:hover { color: #2563eb; }
  }

  :deep(.ant-card-actions) {
    li {
      a {
        color: #3b82f6;
        transition: all 0.2s;

        &:hover {
          color: #2563eb;
        }

        &.text-danger {
          color: #ef4444;

          &:hover {
            color: #dc2626;
          }
        }
      }
    }
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
