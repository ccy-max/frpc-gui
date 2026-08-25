<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '@/stores/app';
import { PlusOutlined, DesktopOutlined, CheckCircleOutlined, PlayCircleOutlined, PauseCircleOutlined, ReloadOutlined } from '@ant-design/icons-vue';
import { message, Modal } from 'ant-design-vue';

const appStore = useAppStore();

const searchKeyword = ref('');
const modalVisible = ref(false);
const editingId = ref<string | null>(null);
const form = ref<any>({
  id: '', name: '', serverAddr: '', serverPort: 7000, token: '', tlsEnable: false, enabled: true,
});

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

// 启动服务器
async function startServer(server: any) {
  try {
    await appStore.startServer(server.id);
    message.success(`服务器 ${server.name} 已启动`);
  } catch (e: any) {
    message.error(`启动失败：${e.message}`);
  }
}

// 停止服务器
async function stopServer(server: any) {
  try {
    await appStore.stopServer(server.id);
    message.success(`服务器 ${server.name} 已停止`);
  } catch (e: any) {
    message.error(`停止失败：${e.message}`);
  }
}

// 重启服务器
async function restartServer(server: any) {
  try {
    await appStore.restartServer(server.id);
    message.success(`服务器 ${server.name} 已重启`);
  } catch (e: any) {
    message.error(`重启失败：${e.message}`);
  }
}

function openAdd() {
  editingId.value = null;
  form.value = { id: Date.now().toString(), name: '', serverAddr: '', serverPort: 7000, token: '', tlsEnable: false, enabled: true };
  modalVisible.value = true;
}

function openEdit(server: any) {
  editingId.value = server.id;
  form.value = { ...server };
  modalVisible.value = true;
}

function handleSave() {
  if (!form.value.name || !form.value.serverAddr) {
    message.warning('请填写必填项');
    return;
  }
  if (editingId.value) {
    appStore.updateServer(editingId.value, form.value);
    message.success('保存成功');
  } else {
    appStore.addServer(form.value);
    message.success('添加成功');
  }
  modalVisible.value = false;
}

async function handleDelete(server: any) {
  // 检查是否有关联的代理
  const relatedProxies = appStore.proxies.filter(p => p.server_id === server.id);
  
  Modal.confirm({
    title: '确认删除',
    content: relatedProxies.length > 0 
      ? `该服务器下有关联的 ${relatedProxies.length} 个代理，删除后这些代理将失去关联。确定要继续吗？`
      : `确定要删除服务器 "${server.name}" 吗？`,
    okText: '确认删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
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
      message.success('删除成功');
    },
  });
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
        <ServerOutlined class="header-icon" />
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
          <!-- 进程控制按钮 -->
          <template v-if="getServerStatus(server.id)?.running">
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
          <a-popconfirm title="确定删除？" @confirm="handleDelete(server)">
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
      <a-form :model="form" layout="vertical">
        <a-form-item label="名称" required>
          <a-input v-model:value="form.name" placeholder="请输入服务器名称" />
        </a-form-item>
        <a-form-item label="服务器地址" required>
          <a-input v-model:value="form.serverAddr" placeholder="例如：127.0.0.1" />
        </a-form-item>
        <a-form-item label="端口" required>
          <a-input-number v-model:value="form.serverPort" :min="1" :max="65535" style="width: 100%" />
        </a-form-item>
        <a-form-item label="令牌">
          <a-input-password v-model:value="form.token" placeholder="请输入令牌" />
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
